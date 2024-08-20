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
} from 'thememirror'

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
        default:
            return smoothy
    }
}