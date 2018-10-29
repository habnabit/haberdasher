declare module 'matrix-js-sdk' {
    import { EventEmitter } from 'events'

    interface Promise {
        done(callback: () => void): void
    }

    interface Client extends EventEmitter {
        login(...args: any[]): any
        startClient(options?: {
            disablePresence?: boolean
            lazyLoadMembers?: boolean
        }): void
        joinRoom(roomId: string): Promise
        publicRooms(): any
        getDevices(): any
        getUserId(): string
        getUser(userid: string): any
        getAccountData(typ: string): any
    }

    export function createClient(options: {
        baseUrl: string
        accessToken: string
        userId: string
    }): Client
    function createClient(baseUrl: string): Client
}
