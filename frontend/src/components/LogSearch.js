import React, { useState, useEffect } from "react";
import axios from "axios";
import { Select, Button, Row, Col, Typography, Popover, Input, Tag, message, List, Tooltip } from 'antd';
import DateRangePicker from './DateRangePicker';  // Importing the DateRangePicker component
import ContextDisplay from "./ContextDisplay"; // 导入 ContextDisplay 组件
import './LogSearch.css'; // Custom styles

const { Option } = Select;
const { Text } = Typography;

const LogSearch = () => {
    const [indices, setIndices] = useState([]);
    const [es_index, setEsIndex] = useState("");
    const [filters, setFilters] = useState({
        hostname: '',
        service: '',
        basename: '',
        dir: '',
        datetime: null
    });
    const [availableFilters, setAvailableFilters] = useState({
        hostname: [],
        service: [],
        basename: [],
    });
    const [loading, setLoading] = useState(false);
    const [filterLoading, setFilterLoading] = useState(false);
    const [searchQuery, setSearchQuery] = useState("");
    const [filterTag, setFilterTag] = useState("");
    const [results, setResults] = useState([]);
    const [popoverVisible, setPopoverVisible] = useState(false);
    const [startTime, setStartTime] = useState("");
    const [endTime, setEndTime] = useState("");
    const [contextData, setContextData] = useState([]);
    const [filterData, setFilterData] = useState({});  // Storing filter data from /discover_node

    useEffect(() => {
        setLoading(true);
        axios.get("http://localhost:8080/get_indices")
            .then(response => {
                const indexList = response.data.indices;
                setIndices(indexList);
                if (indexList.length > 0) {
                    setEsIndex(indexList[0]);
                }
            })
            .finally(() => setLoading(false));
    }, []);

    // Fetching the filter data from discover_node and setting filterData
    useEffect(() => {
        axios.get("http://localhost:8080/discover_node")
            .then(response => {
                setFilterData(response.data); // Save the entire response

                // Extract hostname, service, and log_files from filterData to set in availableFilters
                const hostnameList = Object.keys(response.data); // localhost:9002, localhost:9003
                setAvailableFilters(prevFilters => ({
                    ...prevFilters,
                    hostname: hostnameList,
                    service: [],
                    basename: []
                }));

                // If there is already a selected hostname, we can fill in service options
                if (filters.hostname) {
                    const selectedHostnameData = response.data[filters.hostname];
                    const serviceList = selectedHostnameData.services.map(service => service.service_type);
                    setAvailableFilters(prevFilters => ({
                        ...prevFilters,
                        service: serviceList,
                        basename: [] // Reset basename when service or hostname changes
                    }));
                }
            })
            .catch(error => message.error("Failed to fetch filter data"));
    }, []);

    /**
    const fetchFieldOptions = (field) => {
        setFilterLoading(true);
        axios.get(`http://localhost:8080/indices?index_pattern=${es_index}&field=${field}`)
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
            fetchFieldOptions('basename.keyword');
        }
    }, [es_index]);
     **/
    const handleFilterChange = (value, field) => {
        setFilters(prevFilters => ({
            ...prevFilters,
            [field]: value
        }));

        // Update service and basename when hostname or service changes
        if (field === 'hostname') {
            const selectedHostnameData = filterData[value]; // Get services for the selected hostname
            const selectedServices = selectedHostnameData.services.map(service => service.service_type);
            setAvailableFilters(prevFilters => ({
                ...prevFilters,
                service: selectedServices,
                basename: []  // Reset basename when hostname or service changes
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

    const handleAddFilter = () => {
        const filterContent = [
            filters.hostname ? `hostname: ${filters.hostname}` : '',
            filters.service ? `service: ${filters.service}` : '',
            filters.basename ? `basename: ${filters.basename}` : '',
            filters.dir ? `dir: ${filters.dir}` : '',
        ]
            .filter(Boolean)
            .join(" AND ");

        setFilterTag(filterContent);
        setPopoverVisible(false);
    };

    const handleSearch = () => {
        if (!searchQuery || !es_index || !filters.hostname || !filters.service || !filters.basename || !startTime || !endTime) {
            message.error("All fields are required.");
            return;
        }

        axios.post("http://localhost:8080/search", {
            keyword: searchQuery,
            es_index: es_index,
            hostname: filters.hostname,
            service: filters.service,
            basename: filters.basename,
            start_time: startTime,
            end_time: endTime,
        }).then(response => {
            setResults(response.data.results);
        });
    };

    const handleContextClick = (timestamp) => {
        const date = new Date(timestamp);
        if (isNaN(date.getTime())) {
            message.error("Invalid timestamp.");
            return;
        }

        let milliseconds = date.getMilliseconds();
        if (milliseconds > 900) {
            date.setSeconds(date.getSeconds() + 1);
            date.setMilliseconds(0);
        } else if (milliseconds < 100) {
            date.setSeconds(date.getSeconds() - 1);
            date.setMilliseconds(1000);
        }

        const startTime = new Date(date.getTime() - 100);
        const endTime = new Date(date.getTime() + 100);

        const formattedStartTime = startTime.toISOString();
        const formattedEndTime = endTime.toISOString();

        const requestData = {
            start_time: formattedStartTime,
            end_time: formattedEndTime,
            es_index: es_index,
            hostname: filters.hostname,
            service: filters.service,
            basename: filters.basename
        };

        axios.post("http://localhost:8080/get_log_context", requestData)
            .then(response => {
                setContextData(response.data.log_context);
            })
            .catch(error => message.error("Failed to fetch log context"));
    };

    const handleTagDelete = () => {
        setFilterTag("");
    };

    const handleDateChange = (formattedStart, formattedEnd) => {
        setStartTime(formattedStart);
        setEndTime(formattedEnd);
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
            <Button type="primary" onClick={handleAddFilter} style={{ marginTop: 10 }}>
                Add Filter
            </Button>
        </div>
    );

    return (
        <div className="log-search-container" style={{ width: '100%' }}>
            <Row gutter={[16, 16]} align="middle" style={{ display: 'flex', justifyContent: 'space-between' }}>
                <Col style={{ width: '200px' }}>
                    <Select
                        value={es_index}
                        onChange={(value) => setEsIndex(value)}
                        style={{ width: '100%' }}
                        loading={loading}
                    >
                        {indices.map((index, idx) => (
                            <Option key={idx} value={index}>
                                {index}
                            </Option>
                        ))}
                    </Select>
                </Col>

                <Col style={{ width: '110px' }}>
                    <Popover
                        content={filterPopoverContent}
                        title="Select Filters"
                        trigger="click"
                        placement="bottomLeft"
                        visible={popoverVisible}
                        onVisibleChange={(visible) => setPopoverVisible(visible)}
                        overlayStyle={{ width: 400 }}
                    >
                        <Button type="dashed">Add Filter</Button>
                    </Popover>
                </Col>

                <Col style={{ flex: '1 1 10%' }}>
                    <Input
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search..."
                        style={{ height: '32px' }}
                    />
                </Col>

                <Col style={{ width: '250px' }}>
                    <DateRangePicker onDateChange={handleDateChange} />
                </Col>

                <Col style={{ width: '110px' }}>
                    <Button type="primary" onClick={handleSearch}>Search</Button>
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
                {startTime && endTime && (
                    <Tag color="green" style={{ marginBottom: '8px' }}>
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

            <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
                <List
                    itemLayout="horizontal"
                    dataSource={results}
                    renderItem={item => (
                        <List.Item style={{ fontSize: '12px', padding: '8px' }} onClick={() => handleContextClick(item.timestamp)}>
                            <List.Item.Meta
                                title={
                                    <Tooltip title={`Basename: ${item.basename} \nHostname: ${item.hostname} \nTimestamp: ${item.timestamp}`}>
                                        <Text strong>{item.message.replace(/"/g, "")}</Text>
                                    </Tooltip>
                                }
                            />
                        </List.Item>
                    )}
                />
            </div>
            <ContextDisplay contextData={contextData} />
        </div>
    );
};

export default LogSearch;
