import React, { useState, useEffect } from "react";
import axios from "axios";
import { Select, Button, Row, Col, Typography, Popover, Tag, message } from 'antd';
import './LogSearch.css'; // Custom styles

const { Option } = Select;
const { Text } = Typography;

const FilterForUpload = () => {
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

    // Fetching the filter data from discover_node and setting filterData
    useEffect(() => {
        axios.get("http://localhost:8080/discover_node")
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

    const handleAddFilter = () => {
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
            <Button type="primary" onClick={handleAddFilter} style={{ marginTop: 10 }}>
                Upload To Elasticsearch
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
        </div>
    );
};

export default FilterForUpload;
