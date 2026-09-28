<script setup lang="ts">
  import { usersApi } from '@/api'
  import { roles } from '../data/data'
  import { type User, userRoleSchema } from '../data/schema'
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
  import Textarea from '@/components/ui/textarea/Textarea.vue'
  import PasswordInput from '@/components/PasswordInput.vue'
  import SelectDropdown from '@/components/SelectDropdown.vue'
  import { nullableInput } from '@/lib/utils'
  import { toTypedSchema } from '@vee-validate/zod'
  import { useForm } from 'vee-validate'
  import { computed, ref } from 'vue'
  import { toast } from 'vue-sonner'
  import { z } from 'zod'

  const open = defineModel<boolean>()

  type UserActionDialogProps = {
    user?: User
  }

  const props = defineProps<UserActionDialogProps>()

  const emit = defineEmits(['updated'])

  const isEdit = computed(() => !!props.user)

  const formSchema = toTypedSchema(
    z
      .object({
        name: z.string().min(1, 'Name is required.'),
        email: z.string().email('Email is required.'),
        role: nullableInput(userRoleSchema, 'Role is required.'),
        notes: z.string().nullable(),
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

  const { handleSubmit, isFieldTouched } = useForm({
    validationSchema: formSchema,
    initialValues: isEdit.value
      ? {
          ...props.user,
          password: '',
          confirmPassword: '',
          isEdit: isEdit.value,
        }
      : {
          name: '',
          email: '',
          role: null,
          notes: null,
          password: '',
          confirmPassword: '',
          isEdit: isEdit.value,
        },
  })

  const isLoading = ref(false)
  const { addUser, updateUser } = usersApi
  const onSubmit = handleSubmit(async (values) => {
    try {
      isLoading.value = true
      if (isEdit.value) {
        await updateUser({ ...props.user!, ...values })
      } else {
        await addUser({ ...values } as unknown as User)
      }
      open.value = false
      emit('updated')
    } catch (err) {
      toast.error(err instanceof Error ? `${err.name}: ${err.message}` : String(err))
    } finally {
      isLoading.value = false
    }
  })

  const isPasswordTouched = computed(() => isFieldTouched('password'))
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
          @submit.prevent="onSubmit"
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
                  placeholder="john.doe@email.com"
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
            name="notes"
            v-slot="{ componentField }"
          >
            <FormItem class="grid grid-cols-6 items-center space-y-0 gap-x-4 gap-y-1">
              <FormLabel class="col-span-2 text-end">Notes</FormLabel>
              <FormControl>
                <Textarea
                  placeholder="Add a note"
                  class="col-span-4 resize-none"
                  v-bind="componentField"
                />
              </FormControl>
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
          :disabled="isLoading"
        >
          Save changes
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
