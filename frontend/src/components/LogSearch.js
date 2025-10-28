import React, {useState, useEffect, useRef} from "react";
import axios from "axios";
import {Select, Button, Row, Col, Typography, Popover, Input, Tag, message, List, Tooltip} from 'antd';
import DateRangePicker from './DateRangePicker';  // Importing the DateRangePicker component
import ContextDisplay from "./ContextDisplay"; // 导入 ContextDisplay 组件
import './LogSearch.css'; // Custom styles

const {Option} = Select;
const {Text} = Typography;

const LogSearch = () => {
    const socketRef = useRef(null);

    const [indices, setIndices] = useState([]);
    const [es_index, setEsIndex] = useState("");
    const [filters, setFilters] = useState({
        hostname: "",
        service: "",
    });
    const [availableFilters, setAvailableFilters] = useState({
        hostname: [],
        service: [],
    });
    const [popoverVisible, setPopoverVisible] = useState(false);
    const [filterLoading, setFilterLoading] = useState(false);

    const [loading, setLoading] = useState(false);
    const [searchQuery, setSearchQuery] = useState("");
    const [results, setResults] = useState([]);
    const [startTime, setStartTime] = useState("");
    const [endTime, setEndTime] = useState("");
    const [contextData, setContextData] = useState("");

    const API_BASE_URL = "http://127.0.0.1:8080"

    useEffect(() => {
        refreshElasticSearch();
    }, []);

    const fetchFieldOptions = (field) => {
        setFilterLoading(true);
        axios.get(`${API_BASE_URL}/indices?index_pattern=${es_index}&field=${field}`)
            .then(response => {
                const fieldName = field.split('.')[0];
                setAvailableFilters(prevFilters => ({
                    ...prevFilters,
                    [fieldName]: response.data.unique_services || []
                }));
            })
            .finally(() => setFilterLoading(false));
    };

    useEffect(() => {
        if (es_index) {
            fetchFieldOptions('hostname.keyword');
            fetchFieldOptions('service.keyword');
        }
    }, [es_index]);

    const handleFilterChange = (value, field) => {
        setFilters(prevFilters => ({
            ...prevFilters,
            [field]: value
        }));
    };

    const handleAddFilter = () => {
        setPopoverVisible(false);
    };

    const handleSearch = () => {
        if (!searchQuery || !es_index) {
            message.error("All fields are required.");
            return;
        }

        // 调用 API 搜索，传递 es_index 代替 selectedIndex
        axios.post(`${API_BASE_URL}/keyword_search`, {
            keyword: searchQuery,
            es_index: es_index,  // 传递 es_index
        }).then(response => {
            setResults(response.data.results);
        });
    };

    const refreshElasticSearch = () => {
        setLoading(true);
        axios.get(`${API_BASE_URL}/get_indices`)
            .then(response => {
                const indexList = response.data.indices;
                setIndices(indexList);
                if (indexList.length > 0) {
                    setEsIndex(indexList[0]);
                }
            })
            .finally(() => setLoading(false));
    };

    const splitLogMessage = (logMessage) => {
        // Split the log message into words (split by spaces)
        const words = logMessage.split(" ");
        const selectedWords = words.slice(0, 4);
        // Join the selected words back into a string
        return selectedWords.join(" ");
    }


    const handleContextClick = (item) => {
        if (socketRef.current && socketRef.current.readyState !== WebSocket.CLOSED) {
            socketRef.current.close();
        }
        // Create a new WebSocket connection each time
        socketRef.current = new WebSocket(`ws://${item.hostname}`);
        socketRef.current.onopen = () => {
            console.log("socketRef onopen:");  // Handle the response from the server
            let sp = splitLogMessage(item.message);
            console.log("socketRef open, splitLogMessage: ",sp);
            const messagePayload = {
                cmd: "file_grep",
                filter_strings: [sp],
                file_path:  item.file_name,
                context_line:  4,
            };
            socketRef.current.send(JSON.stringify(messagePayload));  // Send the message to the WebSocket server
            console.log("Message sent:", messagePayload);
        };

        socketRef.current.onmessage = (event) => {
            console.log("Message received:", event.data);  // Handle the response from the server
            setContextData(event.data);
        };

        socketRef.current.onerror = (error) => {
            console.error("WebSocket error:", error);
            message.error("Error occurred while connecting to WebSocket.");
        };

        socketRef.current.onclose = () => {
            console.log("WebSocket connection closed.");
        };
    };




    const handleDateChange = (formattedStart, formattedEnd) => {
        setStartTime(formattedStart);
        setEndTime(formattedEnd);
    };

    const filterPopoverContent = (
        <div>
            <Row gutter={[8, 8]} style={{marginBottom: 8}}>
                <Col span={24}>
                    <Text style={{fontSize: '14px', marginBottom: '4px'}}>Hostname</Text>
                    <Select
                        value={filters.hostname}
                        onChange={(value) => handleFilterChange(value, 'hostname')}
                        placeholder="Select Hostname"
                        style={{width: '100%', height: 32}}
                    >
                        {availableFilters.hostname.map((hostname, idx) => (
                            <Option key={idx} value={hostname}>
                                {hostname}
                            </Option>
                        ))}
                    </Select>
                </Col>

                <Col span={24}>
                    <Text style={{fontSize: '14px', marginBottom: '4px'}}>Service</Text>
                    <Select
                        value={filters.service}
                        onChange={(value) => handleFilterChange(value, 'service')}
                        placeholder="Select Service"
                        style={{width: '100%', height: 32}}
                    >
                        {availableFilters.service.map((service, idx) => (
                            <Option key={idx} value={service}>
                                {service}
                            </Option>
                        ))}
                    </Select>
                </Col>
            </Row>
            <Button type="primary" onClick={handleAddFilter} style={{marginTop: 10}}>
                Add Filter
            </Button>
        </div>
    );

    return (
        <div className="log-search-container" style={{width: '100%'}}>
            <Col style={{width: '250px', marginBottom: 25}}>
                <Button type="primary" onClick={refreshElasticSearch}>RefreshElastic</Button>
            </Col>

            <Row gutter={[16, 16]} align="middle" style={{display: 'flex', justifyContent: 'space-between'}}>
                <Col style={{width: '200px'}}>
                    <Select
                        value={es_index}
                        onChange={(value) => setEsIndex(value)}
                        style={{width: '100%'}}
                        loading={loading}
                    >
                        {indices.map((index, idx) => (
                            <Option key={idx} value={index}>
                                {index}
                            </Option>
                        ))}
                    </Select>
                </Col>

                <Col style={{width: '110px'}}>
                    <Popover
                        content={filterPopoverContent}
                        title="Select Item For ES Search"
                        trigger="click"
                        placement="bottomLeft"
                        visible={popoverVisible}
                        onVisibleChange={(visible) => setPopoverVisible(visible)}
                        overlayStyle={{width: 400}}
                    >
                        <Button type="dashed">Services</Button>
                    </Popover>
                </Col>

                <Col style={{flex: '1 1 10%'}}>
                    <Input
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search..."
                        style={{height: '32px'}}
                    />
                </Col>


                <Col style={{width: '110px'}}>
                    <Button type="primary" onClick={handleSearch}>Search</Button>
                </Col>
            </Row>

            <div style={{marginTop: 20}}>
                {startTime && endTime && (
                    <Tag color="green" style={{marginBottom: '8px'}}>
                        Start Date: {startTime} | End Date: {endTime}
                        <span
                            style={{
                                marginLeft: 8,
                                cursor: 'pointer',
                                color: 'red',
                            }}
                            onClick={() => {
                                setStartTime('');
                                setEndTime('');
                            }}
                        >
                            X
                        </span>
                    </Tag>
                )}
            </div>

            <div style={{maxHeight: '800px', overflowY: 'auto'}}>
                <List
                    itemLayout="horizontal"
                    dataSource={results}
                    renderItem={item => (
                        <List.Item style={{fontSize: '12px', padding: '2px'}}
                                   onClick={() => handleContextClick(item)}>
                            <List.Item.Meta
                                title={
                                    <Tooltip
                                        title={`file_name: ${item.file_name} \nHostname: ${item.hostname} \nTimestamp: ${item.timestamp}`}>
                                        <Text style={{fontSize: '12px'}}>{item.message.replace(/"/g, "")}</Text>
                                    </Tooltip>
                                }
                            />
                        </List.Item>
                    )}
                />
            </div>
            <ContextDisplay contextData={contextData} title="log context"/>
        </div>
    );
};

export default LogSearch;
