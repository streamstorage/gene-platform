import { configure } from 'vee-validate'

export const setupVeeValidate = () => {
  configure({
    // validateOnBlur: false,        // Disables validation on blur
    validateOnChange: false, // Disables validation on change
    validateOnInput: false, // Disables validation on typing/input
    validateOnModelUpdate: false, // Disables validation on v-model updates
  })
}
