import React, {useState, useEffect, useRef} from "react";
import axios from "axios";
import {Select, Button, Row, Col, Typography, Popover, Input, Tag, message, List, Tooltip} from 'antd';
import DateRangePicker from './DateRangePicker';  // Importing the DateRangePicker component
import ContextDisplay from "./ContextDisplay"; // 导入 ContextDisplay 组件
import './LogSearch.css'; // Custom styles

const {Option} = Select;
const {Text} = Typography;

const LogSearch = () => {
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
    const [contextData, setContextData] = useState([]);

    useEffect(() => {
        refreshElasticSearch();
    }, []);

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
        axios.post("http://localhost:8080/keyword_search", {
            keyword: searchQuery,
            es_index: es_index,  // 传递 es_index
        }).then(response => {
            setResults(response.data.results);
        });
    };

    const refreshElasticSearch = () => {
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
    };

    const handleContextClick = (timestamp) => {
        return;
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
            <Col style={{width: '250px',marginBottom: 25}}>
                {/*<DateRangePicker onDateChange={handleDateChange}/>*/}
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
                        <List.Item style={{fontSize: '12px', padding: '8px'}}
                                   onClick={() => handleContextClick(item.timestamp)}>
                            <List.Item.Meta
                                title={
                                    <Tooltip
                                        title={`file_name: ${item.file_name} \nHostname: ${item.hostname} \nTimestamp: ${item.timestamp}`}>
                                        <Text strong>{item.message.replace(/"/g, "")}</Text>
                                    </Tooltip>
                                }
                            />
                        </List.Item>
                    )}
                />
            </div>
            <ContextDisplay contextData={contextData}/>
        </div>
    );
};

export default LogSearch;
