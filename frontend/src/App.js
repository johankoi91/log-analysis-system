import React, { useState } from 'react';
import LogSearch from './components/LogSearch';  // Existing LogSearch component
import SelectLogFile from './components/./SelectLogFile';
import { Layout, Menu } from 'antd';

const { Sider, Content } = Layout;

function App() {
    // State to manage the active page
    const [activePage, setActivePage] = useState('logSearch'); // Default to 'logSearch'

    // Handle menu item click
    const handleMenuClick = (e) => {
        setActivePage(e.key);
    };

    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Sider width={200} className="site-layout-background">
                <Menu
                    mode="inline"
                    defaultSelectedKeys={['LogFileSelect']}
                    style={{ height: '100%', borderRight: 0 }}
                    onClick={handleMenuClick}
                >
                    <Menu.Item key="LogFileSelect">Log File Select</Menu.Item>
                    <Menu.Item key="ESSearch">ES Keyword Search</Menu.Item>
                </Menu>
            </Sider>
            <Layout style={{ padding: '0 24px 24px' }}>
                <Content
                    style={{
                        padding: 24,
                        margin: 0,
                        minHeight: 280,
                    }}
                >
                    {/* Conditionally render the components based on the active page */}
                    {activePage === 'ESSearch' && <LogSearch />}
                    {activePage === 'LogFileSelect' && <SelectLogFile />}
                </Content>
            </Layout>
        </Layout>
    );
}

export default App;
