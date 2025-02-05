// src/components/DateRangePicker.js
import React, { useState } from 'react';
import { DatePicker, Tag, Typography } from 'antd';  // Correct import
const { RangePicker, message } = DatePicker;  // Correctly use RangePicker from DatePicker
const { Text } = Typography;  // Correctly import Typography

const DateRangePicker = ({ onDateChange }) => {
    const [timeRange, setTimeRange] = useState([null, null]);  // 存储时间范围
    const [formattedStartDate, setFormattedStartDate] = useState("");  // 存储格式化后的开始日期
    const [formattedEndDate, setFormattedEndDate] = useState("");  // 存储格式化后的结束日期

    // 处理日期变化
    const handleDateChange = (dates) => {
        if (!dates || dates.length !== 2) {
            message.warning("Please select both Start Date and End Date.");
            return;
        }

        if (dates && dates[0] && dates[1]) {
            const formattedStart = dates[0].toISOString();
            const formattedEnd = dates[1].toISOString();
            setFormattedStartDate(formattedStart);
            setFormattedEndDate(formattedEnd);

            // 传递更新的时间范围给父组件
            onDateChange(formattedStart, formattedEnd);  // Send both start and end dates back to parent
        }
        setTimeRange(dates);  // 更新选择的时间范围
    };

    return (
        <div>
            {/* RangePicker */}
            <RangePicker
                value={timeRange}
                onChange={handleDateChange}
                format="YYYY-MM-DD HH:mm:ss.SSS"  // 精确到毫秒
                showTime={{ format: 'HH:mm:ss.SSS' }}  // 显示时间并支持毫秒
                style={{ width: '100%' }}
            />
            {/* 显示选择的时间 */}
        </div>
    );
};

export default DateRangePicker;
