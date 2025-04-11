type VSCode = {
    postMessage(message: any): void;
    getState(): any;
    setState(state: any): void;
};

declare const vscode: VSCode;

type TrpServer = {
    name: string;
    url: string;
    headers?: Record<string, string>;
}

type Config = {
    trpServers: TrpServer[];
}

declare const config: Config;

interface AbsAppEvent {
    type: 'txs' | 'config';
    data: any;
}

interface TxsAppEvent extends AbsAppEvent {
    type: 'txs';
    data: Tx[];
}

interface TrpServersAppEvent extends AbsAppEvent {
    type: 'config';
    data: Config;
}

declare type AppEvent = TxsAppEvent | TrpServersAppEvent;