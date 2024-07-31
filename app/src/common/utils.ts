import {open} from '@tauri-apps/api/shell'
import {_alertError} from "~/common/events.ts";

export function goBrowserPage(address: string) {
    open(address)
        .then(() => {
        })
        .catch(e => {
            console.error(e)
            _alertError("Open browser failed: {e}")
        })
}