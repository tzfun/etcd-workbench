import mitt from "mitt";
import {useTheme} from "vuetify";
import {DialogItem, TipsItem} from "~/common/types.ts";

export const events = mitt();

export function _loading(state: boolean) {
    events.emit('loading', state)
}

export function toggleTheme() {
    const theme = useTheme()
    theme.global.name.value = theme.global.current.value.dark ? 'light' : 'dark'
}

export function _confirm(title: string, text: string): Promise<undefined> {
    return new Promise((resolve, reject) => {
        let dialog: DialogItem = {
            value: true,
            content: text,
            title,
            icon: 'mdi-alert-circle-outline',
            iconColor: 'yellow-darken-4',
            buttons: [
                {
                    text: "Cancel",
                    callback: (item: DialogItem) => {
                        item.value = false
                        reject()
                    }
                },
                {
                    text: "Confirm",
                    class: 'text-primary',
                    callback: (item: DialogItem) => {
                        item.value = false
                        resolve(undefined)
                    }
                }
            ]
        }

        events.emit('dialog', dialog)
    })

}

export function _confirmSystem(text: string): Promise<undefined> {
    return _confirm('System', text)
}

export function _dialogContent(content: string) {
    let dialog: DialogItem = {
        value: true,
        title:'Display Content',
        content: content,
        maxWidth: 1200,
        closeBtn: true
    }

    events.emit('dialog', dialog)
}

export function _alertError(text: string) {
    let dialog: DialogItem = {
        value: true,
        title: "Error",
        content: text,
        icon: 'mdi-alert-circle-outline',
        iconColor: "red",
        buttons: [
            {
                text: "Close",
                callback: (item: DialogItem) => {
                    item.value = false
                }
            }
        ]
    }

    events.emit('dialog', dialog)
}

export function _tipError(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-alert-circle-outline',
        class: 'bg-red-lighten-1'
    }

    events.emit('tip', tip)
}

export function _tipWarn(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-alert-circle',
        class: 'bg-orange-darken-1'
    }

    events.emit('tip', tip)
}

export function _tipSuccess(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-check',
        class: 'bg-green-lighten-1'
    }

    events.emit('tip', tip)
}

export function _tipInfo(text: string) {
    let tip: TipsItem = {
        value: true,
        content: text,
        timeout: 4000,
        icon: 'mdi-lightbulb-on-40',
        class: 'bg-secondary'
    }

    events.emit('tip', tip)
}