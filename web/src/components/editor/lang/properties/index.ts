import { StreamLanguage } from '@codemirror/language'
import { properties } from '@codemirror/legacy-modes/mode/properties'

const propertiesLanguage = () => StreamLanguage.define(properties)
export default propertiesLanguage
