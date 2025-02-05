import React from "react";
import { List, Typography, Card } from "antd";

const { Text } = Typography;

const ContextDisplay = ({ contextData }) => {
    // 确保 contextData 是一个数组，避免 spread 错误
    const data = Array.isArray(contextData) ? contextData : [];

    // 如果没有上下文数据，直接返回 null
    if (data.length === 0) {
        return null;
    }

    return (
        <Card title="Log Context" style={{ marginTop: 20 }}>
            <List
                itemLayout="horizontal"
                dataSource={data}  // 使用正确的数组
                renderItem={item => (
                    <List.Item style={{ padding: 0 }}>
                        <List.Item.Meta
                            title={<Text
                                strong
                                style={{
                                    fontSize: '12px',      // 设置字体大小
                                    color: '#888',         // 设置灰色字体颜色
                                    lineHeight: '18px'     // 设置较小的行高
                                }}>
                                {item.message.replace(/"/g, "")}
                            </Text>}
                            // description={
                            //     <Text
                            //         style={{
                            //             fontSize: '12px',      // 设置字体大小
                            //             color: '#888',         // 设置灰色字体颜色
                            //             lineHeight: '18px'     // 设置较小的行高
                            //         }}>
                            //         {`Hostname: ${item.hostname}, Basename: ${item.basename}, Timestamp: ${item.timestamp}`}
                            //     </Text>
                            // }
                        />
                    </List.Item>
                )}
            />
        </Card>
    );
};

export default ContextDisplay;
