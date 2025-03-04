import React, {useState, useEffect, useRef} from "react";
import axios from "axios";
import { Select, Button, Row, Col, Typography, Popover, Tag, message, Input, List} from 'antd';
import './LogSearch.css'; // Custom styles
import ContextDisplay from "./ContextDisplay"; // 导入 ContextDisplay 组件

const { Option } = Select;
const { Text } = Typography;
const API_BASE_URL = "http://10.62.0.93:8080"

const SelectLogFile = () => {
    // Ref to hold WebSocket connection
    const socketRef = useRef(null);
    const [filters, setFilters] = useState({
        hostname: "",
        service:  "",
        basename: "",
        dir: ""
    });
    const [availableFilters, setAvailableFilters] = useState({
        hostname: [],
        service: [],
        basename: [],
    });
    const [filterData, setFilterData] = useState({});  // Storing filter data from /discover_node
    const [popoverVisible, setPopoverVisible] = useState(false);
    const [filterTag, setFilterTag] = useState("");  // Store the filter tag

    const [filterStrings, setFilterStrings] = useState({
        keyword1: "",
        keyword2: ""
    });

    // New state to store WebSocket response data
    const [serverResponse, setServerResponse] = useState("");

    const handleKeywordChange = (e, keyword) => {
        setFilterStrings({
            ...filterStrings,
            [keyword]: e.target.value
        });
    };

    // Fetching the filter data from discover_node and setting filterData
    useEffect(() => {
        axios.get(`${API_BASE_URL}/discover_node`)
            .then(response => {
                setFilterData(response.data);

                const hostnameList = Object.keys(response.data);
                setAvailableFilters(prevFilters => ({
                    ...prevFilters,
                    hostname: hostnameList,
                    service: [],
                    basename: []
                }));

                if (filters.hostname) {
                    const selectedHostnameData = response.data[filters.hostname];
                    const serviceList = selectedHostnameData.services.map(service => service.service_type);
                    setAvailableFilters(prevFilters => ({
                        ...prevFilters,
                        service: serviceList,
                        basename: []
                    }));
                }
            })
            .catch(error => message.error("Failed to fetch filter data"));
    }, [filters.hostname]);

    const handleFilterChange = (value, field) => {
        setFilters(prevFilters => ({
            ...prevFilters,
            [field]: value
        }));

        if (field === 'hostname') {
            const selectedHostnameData = filterData[value];
            const selectedServices = selectedHostnameData.services.map(service => service.service_type);
            setAvailableFilters(prevFilters => ({
                ...prevFilters,
                service: selectedServices,
                basename: []
            }));
        }

        if (field === 'service') {
            const selectedServiceData = filterData[filters.hostname].services.find(service => service.service_type === value);
            setAvailableFilters(prevFilters => ({
                ...prevFilters,
                basename: selectedServiceData ? selectedServiceData.log_files : []
            }));

            setFilters(prevFilters => ({
                ...prevFilters,
                dir: selectedServiceData ? selectedServiceData.dir : null
            }));
        }
    };

    const handleAddTag = () => {
        const filterContent = [
            filters.hostname ? `hostname: ${filters.hostname}` : '',
            filters.service ? `service: ${filters.service}` : '',
            filters.basename ? `basename: ${filters.basename}` : '',
            filters.dir ? `dir: ${filters.dir}` : '',
        ]
            .filter(Boolean)
            .join(" AND ");

        setFilterTag(filterContent);  // Set the filter tag
        setPopoverVisible(false);
    };

    const grepLogFile = () => {
        if (!filters.hostname || !filters.basename || !filters.dir || !filters.service) {
            message.error("All fields are required.");
            return;
        }
        if (socketRef.current && socketRef.current.readyState !== WebSocket.CLOSED) {
            socketRef.current.close();
        }
        // Create a new WebSocket connection each time
        socketRef.current = new WebSocket(`ws://${filters.hostname}`);

        // Construct filter strings with AND relationship
        const { keyword1, keyword2 } = filterStrings;

        socketRef.current.onopen = () => {
            console.log("socketRef onopen:");  // Handle the response from the server
            const messagePayload = {
                cmd: "file_grep",
                filter_strings: [keyword1,keyword2],
                file_path:  filters.dir + filters.basename,
                context_line: 0,
            };
            socketRef.current.send(JSON.stringify(messagePayload));  // Send the message to the WebSocket server
            console.log("Message sent:", messagePayload);
        };

        socketRef.current.onmessage = (event) => {
            console.log("Message received:", event.data);  // Handle the response from the server
            setServerResponse(event.data);
        };

        socketRef.current.onerror = (error) => {
            console.error("WebSocket error:", error);
            message.error("Error occurred while connecting to WebSocket.");
        };

        socketRef.current.onclose = () => {
            console.log("WebSocket connection closed.");
        };
    }

    const handleTagDelete = () => {
        setFilterTag("");
        setFilters({
            hostname: "",
            service:  "",
            basename: "",
            dir: ""
        });
    };

    const filterPopoverContent = (
        <div>
            <Row gutter={[8, 8]} style={{ marginBottom: 8 }}>
                <Col span={24}>
                    <Text style={{ fontSize: '14px', marginBottom: '4px' }}>Hostname</Text>
                    <Select
                        value={filters.hostname}
                        onChange={(value) => handleFilterChange(value, 'hostname')}
                        placeholder="Select Hostname"
                        style={{ width: '100%', height: 32 }}
                    >
                        {availableFilters.hostname.map((hostname, idx) => (
                            <Option key={idx} value={hostname}>
                                {hostname}
                            </Option>
                        ))}
                    </Select>
                </Col>

                <Col span={24}>
                    <Text style={{ fontSize: '14px', marginBottom: '4px' }}>Service</Text>
                    <Select
                        value={filters.service}
                        onChange={(value) => handleFilterChange(value, 'service')}
                        placeholder="Select Service"
                        style={{ width: '100%', height: 32 }}
                    >
                        {availableFilters.service.map((service, idx) => (
                            <Option key={idx} value={service}>
                                {service}
                            </Option>
                        ))}
                    </Select>
                </Col>

                <Col span={24}>
                    <Text style={{ fontSize: '14px', marginBottom: '4px' }}>Base-Name</Text>
                    <Select
                        value={filters.basename}
                        onChange={(value) => handleFilterChange(value, 'basename')}
                        placeholder="Select Log Base-Name"
                        style={{ width: '100%', height: 32 }}
                    >
                        {availableFilters.basename.map((basename, idx) => (
                            <Option key={idx} value={basename}>
                                {basename}
                            </Option>
                        ))}
                    </Select>
                </Col>
            </Row>
            <Button type="primary" onClick={handleAddTag} style={{ marginTop: 10 }}>
                Show which log selected
            </Button>
        </div>
    );

    return (
        <div className="log-search-container" style={{ width: '100%' }}>
            <Row gutter={[16, 16]} align="middle" style={{ display: 'flex', justifyContent: 'space-between' }}>
                <Col style={{ width: '200px' }}>
                    <Popover
                        content={filterPopoverContent}
                        title="Choose Log"
                        trigger="click"
                        placement="bottomLeft"
                        visible={popoverVisible}
                        onVisibleChange={(visible) => setPopoverVisible(visible)}
                        overlayStyle={{ width: 400 }}
                    >
                        <Button type="dashed">Choose Logs</Button>
                    </Popover>
                </Col>
            </Row>

            <div style={{ marginTop: 20 }}>
                {filterTag && (
                    <Tag color="blue" style={{ marginBottom: '8px' }}>
                        {filterTag}
                        <span
                            style={{
                                marginLeft: 8,
                                cursor: 'pointer',
                                color: 'red',
                            }}
                            onClick={handleTagDelete}
                        >
                            X
                        </span>
                    </Tag>
                )}
            </div>

            {/* New input fields for the keywords */}
            <Row gutter={16} style={{ marginTop: '20px' }}>
                <Col span={11}>
                    <Input
                        placeholder="Enter first keyword"
                        value={filterStrings.keyword1}
                        onChange={(e) => handleKeywordChange(e, "keyword1")}
                    />
                </Col>
                <Col span={2} style={{ textAlign: 'center', paddingTop: '10px' }}>
                    <Text>AND</Text>
                </Col>
                <Col span={11}>
                    <Input
                        placeholder="Enter second keyword"
                        value={filterStrings.keyword2}
                        onChange={(e) => handleKeywordChange(e, "keyword2")}
                    />
                </Col>
                <Col span={11}>
                    <Button type="primary" onClick={grepLogFile} style={{ marginTop: 10 }}>
                        Grep
                    </Button>
                </Col>
            </Row>
            <ContextDisplay contextData={serverResponse} title="result"/>
        </div>
    );
};

export default SelectLogFile;
