import React, { useState } from 'react';
import LogSearch from './components/LogSearch';
import SelectLogFile from './components/SelectLogFile';
import { Layout, Menu } from 'antd';

const { Sider, Content } = Layout;

function App() {
    const [activePage, setActivePage] = useState('LogFileSelect'); // Default to 'LogFileSelect'

    const handleMenuClick = (e) => {
        setActivePage(e.key);
    };

    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Sider width={200} className="site-layout-background">
                <Menu
                    mode="inline"
                    selectedKeys={[activePage]} // Keep track of the selected page
                    style={{ height: '100%', borderRight: 0 }}
                    onClick={handleMenuClick}
                >
                    <Menu.Item key="LogFileSelect">Log File Operation</Menu.Item>
                    <Menu.Item key="LogSearch">Log Search</Menu.Item>
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
                    {/* Use visibility and display to control the visibility of components */}
                    <div style={{ display: activePage === 'LogFileSelect' ? 'block' : 'none' }}>
                        <SelectLogFile />
                    </div>
                    <div style={{ display: activePage === 'LogSearch' ? 'block' : 'none' }}>
                        <LogSearch />
                    </div>
                </Content>
            </Layout>
        </Layout>
    );
}

export default App;
