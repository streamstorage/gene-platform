<script setup lang="ts">
  import ContentSection from './components/ContentSection.vue'
  import { Button, buttonVariants } from '@/components/ui/button'
  import {
    FormControl,
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
  } from '@/components/ui/form'
  import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
  import { showSubmittedData } from '@/lib/dev'
  import { cn } from '@/lib/utils'
  import { ChevronDownIcon } from '@lucide/vue'
  import { useColorMode } from '@vueuse/core'
  import { toTypedSchema } from '@vee-validate/zod'
  import { useForm } from 'vee-validate'
  import { z } from 'zod'

  const { state } = useColorMode()

  const preferenceFormSchema = toTypedSchema(
    z.object({
      theme: z.enum(['light', 'dark']),
      language: z.enum(['en']),
    })
  )

  const { handleSubmit } = useForm({
    validationSchema: preferenceFormSchema,
    initialValues: {
      theme: state.value,
      language: 'en',
    },
  })

  const onSubmit = handleSubmit((data) => {
    if (data.language != 'en' || data.theme != state.value) {
      showSubmittedData(data)
    }
  })
</script>

<template>
  <ContentSection
    title="Preference"
    desc="Customize your preference for the app. Automatically switch between day
          and night themes."
  >
    <form
      class="space-y-8"
      @submit.prevent="onSubmit"
    >
      <FormField
        name="language"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>Language</FormLabel>
          <div class="relative w-max">
            <FormControl>
              <select
                :class="
                  cn(
                    buttonVariants({ variant: 'outline' }),
                    'w-50 appearance-none font-normal capitalize',
                    'dark:bg-background dark:hover:bg-background'
                  )
                "
                v-bind="componentField"
              >
                <option value="en">English</option>
              </select>
            </FormControl>
            <ChevronDownIcon class="absolute inset-e-3 top-2.5 h-4 w-4 opacity-50" />
          </div>
          <FormDescription class="font-manrope">
            Set the language you want to use in the dashboard.
          </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField
        name="theme"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>Theme</FormLabel>
          <FormDescription> Select the theme for the dashboard. </FormDescription>
          <FormMessage />
          <FormControl>
            <input class="sr-only" />
          </FormControl>
          <RadioGroup
            v-bind="componentField"
            class="grid max-w-md grid-cols-2 gap-8 pt-2"
          >
            <FormItem>
              <FormLabel class="[&:has([data-state=checked])>div]:border-primary">
                <FormControl>
                  <RadioGroupItem
                    value="light"
                    class="sr-only"
                  />
                </FormControl>
                <div class="items-center rounded-md border-2 border-muted p-1 hover:border-accent">
                  <div class="space-y-2 rounded-sm bg-[#ecedef] p-2">
                    <div class="space-y-2 rounded-md bg-white p-2 shadow-xs">
                      <div class="h-2 w-20 rounded-lg bg-[#ecedef]" />
                      <div class="h-2 w-25 rounded-lg bg-[#ecedef]" />
                    </div>
                    <div class="flex items-center space-x-2 rounded-md bg-white p-2 shadow-xs">
                      <div class="h-4 w-4 rounded-full bg-[#ecedef]" />
                      <div class="h-2 w-25 rounded-lg bg-[#ecedef]" />
                    </div>
                    <div class="flex items-center space-x-2 rounded-md bg-white p-2 shadow-xs">
                      <div class="h-4 w-4 rounded-full bg-[#ecedef]" />
                      <div class="h-2 w-25 rounded-lg bg-[#ecedef]" />
                    </div>
                  </div>
                </div>
                <span class="block w-full p-2 text-center font-normal"> Light </span>
              </FormLabel>
            </FormItem>
            <FormItem>
              <FormLabel class="[&:has([data-state=checked])>div]:border-primary">
                <FormControl>
                  <RadioGroupItem
                    value="dark"
                    class="sr-only"
                  />
                </FormControl>
                <div
                  class="items-center rounded-md border-2 border-muted bg-popover p-1 hover:bg-accent hover:text-accent-foreground"
                >
                  <div class="space-y-2 rounded-sm bg-slate-950 p-2">
                    <div class="space-y-2 rounded-md bg-slate-800 p-2 shadow-xs">
                      <div class="h-2 w-20 rounded-lg bg-slate-400" />
                      <div class="h-2 w-25 rounded-lg bg-slate-400" />
                    </div>
                    <div class="flex items-center space-x-2 rounded-md bg-slate-800 p-2 shadow-xs">
                      <div class="h-4 w-4 rounded-full bg-slate-400" />
                      <div class="h-2 w-25 rounded-lg bg-slate-400" />
                    </div>
                    <div class="flex items-center space-x-2 rounded-md bg-slate-800 p-2 shadow-xs">
                      <div class="h-4 w-4 rounded-full bg-slate-400" />
                      <div class="h-2 w-25 rounded-lg bg-slate-400" />
                    </div>
                  </div>
                </div>
                <span class="block w-full p-2 text-center font-normal"> Dark </span>
              </FormLabel>
            </FormItem>
          </RadioGroup>
        </FormItem>
      </FormField>

      <Button type="submit">Update preferences</Button>
    </form>
  </ContentSection>
</template>
