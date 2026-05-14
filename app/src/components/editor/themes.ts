//  See https://thememirror.net/
import {
    amy,
    ayuLight,
    barf,
    bespin,
    birdsOfParadise,
    boysAndGirls,
    clouds,
    cobalt,
    coolGlow,
    dracula,
    espresso,
    noctisLilac,
    rosePineDawn,
    smoothy,
    solarizedLight,
    tomorrow,
} from 'thememirror'
import { vscodeLight, vscodeDark } from '@uiw/codemirror-theme-vscode'
import { darcula } from '@uiw/codemirror-theme-darcula'
import { eclipse } from '@uiw/codemirror-theme-eclipse'

import {Extension} from "@codemirror/state";
import {_useSettings} from "~/common/store.ts";

export {
    amy,
    ayuLight,
    barf,
    bespin,
    birdsOfParadise,
    boysAndGirls,
    clouds,
    cobalt,
    coolGlow,
    dracula,
    espresso,
    noctisLilac,
    rosePineDawn,
    smoothy,
    solarizedLight,
    tomorrow,
    vscodeLight,
    vscodeDark,
    darcula,
    eclipse
}

export function getTheme(appTheme: string): Extension {
    let setting = _useSettings().value;
    let themeName
    if (appTheme == 'dark') {
        themeName = setting.editorDarkTheme
    } else {
        themeName = setting.editorLightTheme
    }
    return getThemeByName(themeName)
}

export function getThemeByName(name: string) {
    switch (name) {
        case 'ayuLight':
            return ayuLight
        case 'clouds':
            return clouds
        case 'espresso':
            return espresso
        case 'noctisLilac':
            return noctisLilac
        case 'rosePineDawn':
            return rosePineDawn
        case 'smoothy':
            return smoothy
        case 'solarizedLight':
            return solarizedLight
        case 'tomorrow':
            return tomorrow
        case 'amy':
            return amy
        case 'barf':
            return barf
        case 'bespin':
            return bespin
        case 'birdsOfParadise':
            return birdsOfParadise
        case 'boysAndGirls':
            return boysAndGirls
        case 'cobalt':
            return cobalt
        case 'coolGlow':
            return coolGlow
        case 'dracula':
            return dracula
        case 'vscodeLight':
            return vscodeLight
        case 'vscodeDark':
            return vscodeDark
        case 'ideaLight':
            return eclipse
        case 'ideaDark':
            return darcula
        default:
            return smoothy
    }
}