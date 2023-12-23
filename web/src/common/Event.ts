const listeners: EventListener[] = []

export type EventListener = (key: string, event?: any) => any | undefined

export function registerEventListener(listener: EventListener) {
    if (listener) {
        listeners.push(listener)
    } else {
        throw Error(`Invalid event listener ${listener}`)
    }
}

export function unregisterEventListener(listener: EventListener): boolean {
    if (listener) {
        let idx = listeners.indexOf(listener)
        if (idx >= 0) {
            listeners.splice(idx, 1)
            return true
        }
    }
    return false
}

export function pushEvent(key: string, event?: any) {
    listeners.forEach(l => l(key, event))
}