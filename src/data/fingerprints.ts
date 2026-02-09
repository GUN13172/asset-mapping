export interface Fingerprint {
    name: string;
    description: string;
    fofa: string;
    hunter: string;
    quake: string;
    category: string;
}

export const fingerprints: Fingerprint[] = [
    {
        name: "致远互联-OA",
        description: "Seeyon OA / A6 / A8",
        fofa: 'app="致远互联-OA"',
        hunter: 'app.name="致远互联-OA"',
        quake: 'app: "致远互联-OA"',
        category: "OA办公系统"
    },
    {
        name: "泛微-协同办公OA",
        description: "e-cology / e-office",
        fofa: 'app="泛微-协同办公OA"',
        hunter: 'app.name="泛微-协同办公OA"',
        quake: 'app: "泛微-协同办公OA"',
        category: "OA办公系统"
    },
    {
        name: "通达OA",
        description: "Office Anywhere",
        fofa: 'app="TD_OA"',
        hunter: 'app.name="通达OA"',
        quake: 'app: "通达OA"',
        category: "OA办公系统"
    },
    {
        name: "ThinkPHP",
        description: "PHP开源框架",
        fofa: 'app="ThinkPHP"',
        hunter: 'app.name="ThinkPHP"',
        quake: 'app: "ThinkPHP"',
        category: "Web框架"
    },
    {
        name: "Shiro",
        description: "Apache Shiro 安全框架",
        fofa: 'app="Apache-Shiro"',
        hunter: 'app.name="Apache-Shiro"',
        quake: 'app: "Apache Shiro"',
        category: "中间件/库"
    },
    {
        name: "海康威视-视频监控",
        description: "Hikvision IP Camera/NVR",
        fofa: 'app="HIKVISION-视频监控"',
        hunter: 'app.name="海康威视-视频监控"',
        quake: 'app: "HIKVISION-视频监控"',
        category: "IoT/监控"
    },
    {
        name: "大华-监控设备",
        description: "Dahua IP Camera",
        fofa: 'app="大华-网络视频监控"',
        hunter: 'app.name="大华-网络视频监控"',
        quake: 'app: "Dahua-ICM"',
        category: "IoT/监控"
    },
    {
        name: "Nginx",
        description: "Web Server",
        fofa: 'app="Nginx"',
        hunter: 'app.name="Nginx"',
        quake: 'app: "Nginx"',
        category: "基础组件"
    },
    {
        name: "MySQL",
        description: "Relational Database",
        fofa: 'protocol="mysql"',
        hunter: 'ip.port="3306"',
        quake: 'service: "mysql"',
        category: "数据库"
    },
    {
        name: "Redis",
        description: "In-memory Database",
        fofa: 'protocol="redis"',
        hunter: 'ip.port="6379"',
        quake: 'service: "redis"',
        category: "数据库"
    },
    {
        name: "泛微-移动端E-Bridge",
        description: "Weaver E-Bridge",
        fofa: 'app="泛微-云桥e-Bridge"',
        hunter: 'app.name="泛微-云桥e-Bridge"',
        quake: 'app: "泛微-云桥e-Bridge"',
        category: "OA办公系统"
    },
    {
        name: "Nacos",
        description: "阿里巴巴配置中心/注册中心",
        fofa: 'app="Nacos"',
        hunter: 'app.name="Nacos"',
        quake: 'app: "Nacos"',
        category: "中间件/库"
    },
    {
        name: "Jenkins",
        description: "自动化构建服务器",
        fofa: 'app="Jenkins"',
        hunter: 'app.name="Jenkins"',
        quake: 'app: "Jenkins"',
        category: "开发/运维工具"
    },
    {
        name: "WebLogic",
        description: "Oracle WebLogic Server",
        fofa: 'app="BEA-WebLogic-Server"',
        hunter: 'app.name="Oracle-WebLogic-Server"',
        quake: 'app: "WebLogic"',
        category: "中间件/库"
    },
    {
        name: "Apache-Tomcat",
        description: "Java Servlet 容器",
        fofa: 'app="Apache-Tomcat"',
        hunter: 'app.name="Apache-Tomcat"',
        quake: 'app: "Apache Tomcat"',
        category: "中间件/库"
    },
    {
        name: "Zabbix",
        description: "网络监控管理系统",
        fofa: 'app="Zabbix"',
        hunter: 'app.name="Zabbix"',
        quake: 'app: "Zabbix"',
        category: "监控系统"
    }
];
