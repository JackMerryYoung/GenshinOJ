# IDEA（历史设计稿）

> 本文记录早期 Python Working Load 架构，仅用于设计历史。当前实现是
> `rust_backend/main_backend` 动态加载 Rust 模块；开发说明见 `README.md` 和 `CLAUDE.md`。

基础的就是以一种叫做 Working Load 的东西支撑整个程序

举个例子，Websocket Server Application Loader 就是一个工作负载，然后一般有这几种工作负载：

1. API
    顾名思义就是提供接口的，这里有 Global Message Queue 的部分代码（写的有点混乱邪恶）与 Compilers Manager。
2. Event Loop
    第二种就是事件循环，比如 Chat Server。
3. Apps / Plugins Loader
    比如 Websocket Application Loader 和 Compilers Manager 附属的 Compilers Loader

首先我们来看入口点：

``` py
async def async_main(self):
    print('Server started.')
    with open(self.MODULE_CONFIG_JSON_PATH, 'r') as self.module_config_json_file:
        self.module_config = json.load(self.module_config_json_file)
    
    self.working_loads_config = self.module_config['working_load']
    for working_load_config in self.working_loads_config:
        if working_load_config['enabled']:
            print('Loading {} (id: {})...'.format(working_load_config['name'], working_load_config['id']))
            dependencies_satisfied_flag = True
            for dependency in working_load_config['dependencies']:
                print('Checking dependency: {}'.format(dependency))
                if not dependency in self.working_loads:
                    dependencies_satisfied_flag = False
                    break
            
            if not dependencies_satisfied_flag:
                print('Module {} (id: {}) do not have all its dependencies, so it will be ignored.'.format(working_load_config['name'], working_load_config['id']))
                continue
            
            print('Loaded {} (id: {}).'.format(working_load_config['name'], working_load_config['id']))
            self.working_loads[working_load_config['id']] = working_load_config
            getattr(getattr(importlib.__import__(working_load_config['path']), working_load_config['id']), working_load_config['id'])(self)
    
    await asyncio.wait(self.tasks)
    
    asyncio.get_event_loop().run_forever()
```

这里读取工作负载 json 配置文件，然后依次加载（检查依赖），添加每个负载的实例到管理列表里。（这里我还没有做拓扑排序解决依赖问题，之后会做的）

为了实现负载间调用/通信我写了个 `get_module_instance()` 方法用于获取某一特定工作负载的实例：

``` py
def get_module_instance(self, module_id: str):
    return self.working_loads[module_id]['instance']
```

在这里，工作负载的第二类，即事件循环，在实例化时不仅被要求加入管理列表里，并且要把代表事件循环的 `asyncio.Task` 对象加入主服务的 tasks 这个列表里，方便最后统一执行。

这样做的目的是防止堵塞线程导致我加载不了其他工作负载了。

这里我们规定，客户端发过来的消息前面加个 `on_` 便是我们要调用的回调 callback 函数。

显然一个消息经常要动到多个 WSApp 来处理，为了方便我直接 `getattr()`，若这个 WSApp 没有实现这个消息的处理方法，触发了 `AttributeError` 我直接不管了 `pass` 掉，这样就用 Python 反射机制实现了对于每条消息的处理。

也就是说，我们以后为了添加功能只要写一个 WSApp 就行，大大提高了代码简洁与范式化程度。

特别的，对于每个 WSApp 都得继承一个基类叫 `ws_server_application_protocol`：

``` py
class ws_server_application_protocol:
    """
    Base class for any implementations of additional websocket server applications
    """
    @typing.final
    def __init__(
        self, 
        ws_server_instance: ws_server
    ):
        self.ws_server_instance: ws_server = ws_server_instance
    
    @abc.abstractmethod
    def log(
        self, 
        log: str, 
        log_level: ws_server_log_level = ws_server_log_level.LEVEL_INFO
    ):
        """
        Logging method
        Args:
            log (str): Log information
            log_level (ws_server_log_level): Logging level
        Info:
            It is suggested that you should override this method to distinguish between the official application and yours.
        """
        if log_level is ws_server_log_level.LEVEL_INFO:
            print('[WS_SERVER] [INFO] {}'.format(log))
        if log_level is ws_server_log_level.LEVEL_DEBUG:
            print('[WS_SERVER] [DEBUG] {}'.format(log))
        if log_level is ws_server_log_level.LEVEL_WARNING:
            print('[WS_SERVER] [WARNING] {}'.format(log))
        if log_level is ws_server_log_level.LEVEL_ERROR:
            print('[WS_SERVER] [ERROR] {}'.format(log))
    
    @abc.abstractmethod
    async def on_login(
        self, 
        websocket_protocol: websockets.server.WebSocketServerProtocol, 
        content: dict
    ):
        """
        Callback method `on_login`
        Args:
            websocket_protocol (websockets.server.WebSocketServerProtocol): the protocol of a websocket connection
            username (str): the login username
            password (str): the login password
        Info:
            You need to implement this method to do the specific actions you want whenever a user tries to login.
        """
        self.log('{} tries to login with the password: {}'.format(content['username'], content['password']))

    @abc.abstractmethod
    async def on_quit(
        self, 
        websocket_protocol: websockets.server.WebSocketServerProtocol, 
        content: dict
    ):
        """ 
        Callback method `on_quit`
        Info:
            You need to implement this method to do the specific actions you want whenever a user tries to quit.
        """
        self.log('{} quitted with session token: {}'.format(content['username'], content['session_token']))
```

可以看到他定义了 3 个抽象方法与一个不可重写的被 `@typing.final` 注解了的 `__init__()` 方法；继承这个基类，然后实现对于功能，就是一个 WSApp.

然后，最恶心的部分就搞完了，但是这个设计很棒，不是么？

可以看到一个加载器的基本写法，就是提供一个富含抽象方法的基类，然后让插件或应用编写者继承他，然后我们通过读取配置列表即可加载这个插件或应用，是一种开放式维护方式。

这样做的生态就会发展；本来这个想法是我 1 年前想用在绘版上的，但是居然先用在这里了。
