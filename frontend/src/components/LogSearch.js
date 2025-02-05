// src/components/LogSearch.js
import React, { useState, useEffect } from "react";
import axios from "axios";
import { Select, Button, Row, Col, Typography, Popover, Input, Tag, message, List, Tooltip } from 'antd';
import DateRangePicker from './DateRangePicker';  // Importing the DateRangePicker component
import ContextDisplay from "./ContextDisplay"; // 导入 ContextDisplay 组件
import './LogSearch.css'; // Custom styles
import dayjs from "dayjs";

const { Option } = Select;
const { Text } = Typography;

const LogSearch = () => {
    const [indices, setIndices] = useState([]);
    const [es_index, setEsIndex] = useState("");  // 修改为 es_index
    const [filters, setFilters] = useState({
        hostname: '',
        service: '',
        basename: '',
        datetime: null
    });
    const [availableFilters, setAvailableFilters] = useState({
        hostname: [],
        service: [],
        basename: [],
    });
    const [loading, setLoading] = useState(false);  // 控制 loading 状态
    const [filterLoading, setFilterLoading] = useState(false);  // 控制字段加载状态
    const [searchQuery, setSearchQuery] = useState("");  // 用于存储输入框的内容
    const [filterTag, setFilterTag] = useState("");  // 存储唯一过滤器标签内容
    const [results, setResults] = useState([]);  // 存储搜索结果
    const [popoverVisible, setPopoverVisible] = useState(false);  // 控制 Popover 显示状态

    const [startTime, setStartTime] = useState("");  // 存储开始时间
    const [endTime, setEndTime] = useState("");  // 存储结束时间

    const [contextData, setContextData] = useState([]);

    // 请求索引列表
    useEffect(() => {
        setLoading(true);  // 开始加载
        axios.get("http://localhost:8080/get_indices")
            .then(response => {
                const indexList = response.data.indices;
                setIndices(indexList);
                if (indexList.length > 0) {
                    setEsIndex(indexList[0]);  // 设置 es_index
                }
            })
            .finally(() => {
                setLoading(false);  // 加载完成
            });
    }, []);

    // 根据索引和字段请求可选项
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
            .finally(() => {
                setFilterLoading(false);  // 完成加载
            });
    };

    useEffect(() => {
        if (es_index) {
            fetchFieldOptions('hostname.keyword');
            fetchFieldOptions('service.keyword');
            fetchFieldOptions('basename.keyword');
        }
    }, [es_index]);

    const handleFilterChange = (value, field) => {
        setFilters(prevFilters => ({
            ...prevFilters,
            [field]: value
        }));
    };

    const handleAddFilter = () => {
        // 格式化过滤器为标签内容
        const filterContent = [
            filters.hostname ? `hostname: ${filters.hostname}` : '',
            filters.service ? `service: ${filters.service}` : '',
            filters.basename ? `basename: ${filters.basename}` : '',
        ]
            .filter(Boolean) // 过滤掉空值
            .join(" AND ");

        // 更新过滤器标签（只显示一个标签）
        setFilterTag(filterContent);
        setPopoverVisible(false);  // 关闭 Popover
    };

    const handleSearch = () => {
        // 检查必填项是否存在
        if (!searchQuery) {
            message.error("Keyword is required.");
            return;
        }
        if (!es_index) {
            message.error("ES Index is required.");
            return;
        }
        if (!filters.hostname) {
            message.error("Hostname is required.");
            return;
        }
        if (!filters.service) {
            message.error("Service is required.");
            return;
        }
        if (!filters.basename) {
            message.error("Base-Name is required.");
            return;
        }

        // 检查 start_time 和 end_time 是否为空
        if (!startTime || !endTime) {
            message.error("Start Time and End Time are required.");
            return;
        }

        // 调用 API 搜索，传递 es_index 代替 selectedIndex
        axios.post("http://localhost:8080/search", {
            keyword: searchQuery,
            es_index: es_index,  // 传递 es_index
            hostname: filters.hostname,
            service: filters.service,
            basename: filters.basename,
            start_time: startTime,  // 包括 start_time
            end_time: endTime,  // 包括 end_time
        }).then(response => {
            setResults(response.data.results);  // 设置搜索结果
        });
    };


    const handleContextClick = (timestamp) => {
        console.log("Received timestamp:", timestamp);  // 调试输出

        // 使用 Date 解析 timestamp
        const date = new Date(timestamp);

        // 检查日期是否有效
        if (isNaN(date.getTime())) {
            message.error("Invalid timestamp.");
            return;
        }

        console.log("Parsed Date:", date.toISOString());  // 调试输出

        // 获取毫秒部分
        let milliseconds = date.getMilliseconds();
        console.log("Milliseconds:", milliseconds);  // 输出毫秒部分

        // 如果毫秒部分大于 900ms，向前调整至下一个秒
        if (milliseconds > 900) {
            date.setSeconds(date.getSeconds() + 1);
            date.setMilliseconds(0);  // 设置为 0 毫秒
        }
        // 如果毫秒部分小于 100ms，回退 1秒并设置为 1000ms
        else if (milliseconds < 100) {
            date.setSeconds(date.getSeconds() - 1);
            date.setMilliseconds(1000);  // 设置为上一秒的 1000ms
        }

        // 计算上下浮动 100ms 的时间范围
        const startTime = new Date(date.getTime() - 100);  // 100ms 前
        const endTime = new Date(date.getTime() + 100);    // 100ms 后

        // 格式化时间为 ISO 8601 格式
        const formattedStartTime = startTime.toISOString();
        const formattedEndTime = endTime.toISOString();

        console.log("Start Time:", formattedStartTime);
        console.log("End Time:", formattedEndTime);

        // 组合 API 请求参数
        const requestData = {
            start_time: formattedStartTime,
            end_time: formattedEndTime,
            es_index: es_index, // 从选择的 index 获取
            hostname: filters.hostname, // 从 filters 获取
            service: filters.service,   // 从 filters 获取
            basename: filters.basename  // 从 filters 获取
        };

        // 发送请求获取日志上下文
        axios.post("http://localhost:8080/get_log_context", requestData)
            .then(response => {
                setContextData(response.data.log_context);  // 更新上下文数据
            })
            .catch(error => {
                message.error("Failed to fetch log context.");
            });
    };




    const handleTagDelete = () => {
        // 删除标签时清空过滤器标签
        setFilterTag("");
    };

    const handleDateChange = (formattedStart, formattedEnd) => {
        // 设置 start_time 和 end_time
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
                {/* Left Section: es_index and Add Filter */}
                <Col style={{ width: '200px' }}>
                    <Select
                        value={es_index}
                        onChange={(value) => setEsIndex(value)}  // 修改为 es_index
                        style={{ width: '100%' }}
                        loading={loading}  // 显示加载中状态
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
                        <Button type="dashed">
                            Add Filter
                        </Button>
                    </Popover>
                </Col>

                {/* Center Section: Input */}
                <Col style={{ flex: '1 1 10%' }}>
                    <Input
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search..."
                        style={{ height: '32px' }}
                    />
                </Col>

                {/* Date Picker Section */}
                <Col style={{ width: '250px' }}>
                    <DateRangePicker
                        onDateChange={handleDateChange}
                    />
                </Col>

                {/* Right Section: Search Button */}
                <Col style={{ width: '110px' }}>
                    <Button type="primary" onClick={handleSearch}>
                        Search
                    </Button>
                </Col>
            </Row>

            {/* Filter tags */}
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
                            onClick={handleTagDelete} // 删除标签
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
                            }} // 删除日期标签
                        >
                            X
                        </span>
                    </Tag>
                )}
            </div>

            {/* Display results */}
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
            {/* Context Display */}
            <ContextDisplay contextData={contextData} />
        </div>
    );
};

export default LogSearch;
