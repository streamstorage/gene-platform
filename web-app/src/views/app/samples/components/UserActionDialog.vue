<script setup lang="ts">
  import { roles } from '../data/data'
  import { type User } from '../data/schema'
  import { Button } from '@/components/ui/button'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'
  import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
  import { Input } from '@/components/ui/input'
  import PasswordInput from '@/components/PasswordInput.vue'
  import SelectDropdown from '@/components/SelectDropdown.vue'
  import { showSubmittedData } from '@/lib/dev'
  import { toTypedSchema } from '@vee-validate/zod'
  import { useForm } from 'vee-validate'
  import { computed } from 'vue'
  import { z } from 'zod'

  const open = defineModel<boolean>()

  type UserActionDialogProps = {
    currentRow?: User
  }

  const props = defineProps<UserActionDialogProps>()

  const isEdit = !!props.currentRow

  const formSchema = toTypedSchema(
    z
      .object({
        name: z.string().min(1, 'Name is required.'),
        email: z.string().email('Email is required.'),
        role: z.string().min(1, 'Role is required.'),
        password: z.string().transform((pwd) => pwd.trim()),
        confirmPassword: z.string().transform((pwd) => pwd.trim()),
        isEdit: z.boolean(),
      })
      .refine(
        (data) => {
          if (data.isEdit && !data.password) return true
          return data.password.length > 0
        },
        {
          message: 'Password is required.',
          path: ['password'],
        }
      )
      .refine(
        ({ isEdit, password }) => {
          if (isEdit && !password) return true
          return password.length >= 8
        },
        {
          message: 'Password must be at least 8 characters long.',
          path: ['password'],
        }
      )
      .refine(
        ({ isEdit, password }) => {
          if (isEdit && !password) return true
          return /[a-z]/.test(password)
        },
        {
          message: 'Password must contain at least one lowercase letter.',
          path: ['password'],
        }
      )
      .refine(
        ({ isEdit, password }) => {
          if (isEdit && !password) return true
          return /\d/.test(password)
        },
        {
          message: 'Password must contain at least one number.',
          path: ['password'],
        }
      )
      .refine(
        ({ isEdit, password, confirmPassword }) => {
          if (isEdit && !password) return true
          return password === confirmPassword
        },
        {
          message: "Passwords don't match.",
          path: ['confirmPassword'],
        }
      )
  )
  const form = useForm({
    validationSchema: formSchema,
    initialValues: isEdit
      ? {
          ...props.currentRow,
          password: '',
          confirmPassword: '',
          isEdit,
        }
      : {
          name: '',
          email: '',
          role: '',
          password: '',
          confirmPassword: '',
          isEdit,
        },
  })
  const onSubmit = form.handleSubmit((values) => {
    form.resetForm()
    showSubmittedData(values)
    open.value = false
  })

  const isPasswordTouched = computed(() => form.isFieldTouched('password'))
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-lg">
      <DialogHeader class="text-start">
        <DialogTitle>{{ isEdit ? 'Edit User' : 'Add New User' }}</DialogTitle>
        <DialogDescription>
          {{ isEdit ? 'Update the user here. ' : 'Create new user here. ' }}
          Click save when you&apos;re done.
        </DialogDescription>
      </DialogHeader>
      <div class="h-105 w-[calc(100%+0.75rem)] overflow-y-auto py-1 pe-3">
        <form
          id="user-form"
          @submit="onSubmit"
          class="space-y-4 px-0.5"
        >
          <FormField
            name="name"
            v-slot="{ componentField }"
          >
            <FormItem class="grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1">
              <FormLabel class="col-span-2 text-end"> Name </FormLabel>
              <FormControl>
                <Input
                  placeholder="john_doe"
                  class="col-span-4"
                  v-bind="componentField"
                />
              </FormControl>
              <FormMessage class="col-span-4 col-start-3" />
            </FormItem>
          </FormField>
          <FormField
            name="email"
            v-slot="{ componentField }"
          >
            <FormItem class="grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1">
              <FormLabel class="col-span-2 text-end">Email</FormLabel>
              <FormControl>
                <Input
                  placeholder="john.doe@gmail.com"
                  class="col-span-4"
                  v-bind="componentField"
                />
              </FormControl>
              <FormMessage class="col-span-4 col-start-3" />
            </FormItem>
          </FormField>
          <FormField
            name="role"
            v-slot="{ componentField }"
          >
            <FormItem class="grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1">
              <FormLabel class="col-span-2 text-end">Role</FormLabel>
              <SelectDropdown
                v-bind="componentField"
                placeholder="Select a role"
                class="col-span-4"
                :items="
                  roles.map(({ label, value }) => ({
                    label,
                    value,
                  }))
                "
              />
              <FormMessage class="col-span-4 col-start-3" />
            </FormItem>
          </FormField>
          <FormField
            name="password"
            v-slot="{ componentField }"
          >
            <FormItem class="grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1">
              <FormLabel class="col-span-2 text-end"> Password </FormLabel>
              <FormControl>
                <PasswordInput
                  placeholder="e.g., S3cur3P@ssw0rd"
                  class="col-span-4"
                  v-bind="componentField"
                />
              </FormControl>
              <FormMessage class="col-span-4 col-start-3" />
            </FormItem>
          </FormField>
          <FormField
            name="confirmPassword"
            v-slot="{ componentField }"
          >
            <FormItem class="grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1">
              <FormLabel class="col-span-2 text-end"> Confirm Password </FormLabel>
              <FormControl>
                <PasswordInput
                  :disabled="!isPasswordTouched"
                  placeholder="e.g., S3cur3P@ssw0rd"
                  class="col-span-4"
                  v-bind="componentField"
                />
              </FormControl>
              <FormMessage class="col-span-4 col-start-3" />
            </FormItem>
          </FormField>
        </form>
      </div>
      <DialogFooter>
        <Button
          type="submit"
          form="user-form"
        >
          Save changes
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
