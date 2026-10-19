<script setup lang="ts">
  import { type User } from '@/views/admin/users/data/schema'
  import ContentSection from './components/ContentSection.vue'
  import { Button } from '@/components/ui/button'
  import {
    FormControl,
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
  } from '@/components/ui/form'
  import { Input } from '@/components/ui/input'
  import { Textarea } from '@/components/ui/textarea'
  import PasswordInput from '@/components/PasswordInput.vue'
  import { useAuthStore } from '@/stores'
  import { toTypedSchema } from '@vee-validate/zod'
  import { storeToRefs } from 'pinia'
  import { useForm } from 'vee-validate'
  import { computed, ref, onActivated } from 'vue'
  import { toast } from 'vue-sonner'
  import { z } from 'zod'

  const profileFormSchema = toTypedSchema(
    z
      .object({
        name: z
          .string()
          .min(2, 'Name must be at least 2 characters.')
          .max(30, 'Name must not be longer than 30 characters.'),
        email: z.string().email('Please enter your email.'),
        notes: z.string().max(255).nullable(),
        password: z.string().transform((pwd) => pwd.trim()),
        newPassword: z.string().transform((pwd) => pwd.trim()),
        confirmPassword: z.string().transform((pwd) => pwd.trim()),
      })
      .refine(
        (data) => {
          return data.password.length > 0
        },
        {
          message: 'Password is required.',
          path: ['password'],
        }
      )
      .refine(
        ({ newPassword }) => {
          if (!newPassword) return true
          return newPassword.length >= 8
        },
        {
          message: 'New Password must be at least 8 characters long.',
          path: ['newPassword'],
        }
      )
      .refine(
        ({ newPassword }) => {
          if (!newPassword) return true
          return /[a-z]/.test(newPassword)
        },
        {
          message: 'New Password must contain at least one lowercase letter.',
          path: ['newPassword'],
        }
      )
      .refine(
        ({ newPassword }) => {
          if (!newPassword) return true
          return /\d/.test(newPassword)
        },
        {
          message: 'New Password must contain at least one number.',
          path: ['newPassword'],
        }
      )
      .refine(
        ({ newPassword, confirmPassword }) => {
          if (!newPassword) return true
          return newPassword === confirmPassword
        },
        {
          message: "New Passwords don't match.",
          path: ['confirmPassword'],
        }
      )
  )

  const authStore = useAuthStore()
  const { user } = storeToRefs(authStore)
  const { handleSubmit, isFieldTouched, resetForm } = useForm({
    validationSchema: profileFormSchema,
    initialValues: {
      name: user.value.name,
      email: user.value.email,
      notes: user.value.notes ?? null,
      password: '',
      newPassword: '',
      confirmPassword: '',
    },
  })

  const { updateUserProfile } = authStore
  const onSubmit = handleSubmit(async (data) => {
    try {
      isLoading.value = true
      await updateUserProfile({
        name: data.name,
        email: data.email,
        notes: data.notes,
        password: data.password,
        newPassword: data.newPassword,
        role: 0,
      } as User & { password: string; newPassword: string })

      toast.info('Profile updated')
      resetForm({
        values: {
          name: data.name,
          email: data.email,
          notes: data.notes,
        },
      })
    } catch (err) {
      toast.error(err instanceof Error ? `${err.name}: ${err.message}` : String(err))
    } finally {
      isLoading.value = false
    }
  })

  const isNewPasswordTouched = computed(() => isFieldTouched('newPassword'))
  const isLoading = ref(false)

  onActivated(() => {
    resetForm({
      values: {
        name: user.value.name,
        email: user.value.email,
        notes: user.value.notes,
      },
    })
  })
</script>

<template>
  <ContentSection
    title="Profile"
    desc="This is how others will see you on the site."
  >
    <form
      class="space-y-8"
      id="profile-form"
      @submit.prevent="onSubmit"
    >
      <FormField
        name="name"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>Name</FormLabel>
          <FormControl>
            <Input
              placeholder="shadcn"
              v-bind="componentField"
            />
          </FormControl>
          <FormDescription>
            This is your public display name. It can be your real name or a pseudonym. You can only
            change this once every 30 days.
          </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField
        name="email"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>Email</FormLabel>
          <FormControl>
            <Input
              placeholder="shadcn"
              v-bind="componentField"
            />
          </FormControl>
          <FormDescription>
            This is your email address for sign in. You can only change this once every 30 days.
          </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField
        name="notes"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>Bio</FormLabel>
          <FormControl>
            <Textarea
              placeholder="Tell us a little bit about yourself"
              class="resize-none"
              v-bind="componentField"
            />
          </FormControl>
          <FormDescription> Tell us a little bit about yourself. </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField
        name="password"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>Password</FormLabel>
          <FormControl>
            <PasswordInput
              placeholder="xxxxxxxx"
              v-bind="componentField"
            />
          </FormControl>
          <FormDescription> Please input your password to update the profile. </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>

      <FormField
        name="newPassword"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>New Password</FormLabel>
          <FormControl>
            <PasswordInput
              placeholder="e.g., S3cur3P@ssw0rd"
              v-bind="componentField"
            />
          </FormControl>
          <FormDescription>
            Leave blank if you do not want to change the password.
          </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField
        name="confirmPassword"
        v-slot="{ componentField }"
      >
        <FormItem>
          <FormLabel>New Password</FormLabel>
          <FormControl>
            <PasswordInput
              :disabled="!isNewPasswordTouched"
              placeholder="e.g., S3cur3P@ssw0rd"
              v-bind="componentField"
            />
          </FormControl>
          <FormDescription> Confirm the new password. </FormDescription>
          <FormMessage />
        </FormItem>
      </FormField>

      <Button
        type="submit"
        form="profile-form"
        :disabled="isLoading"
      >
        Update profile
      </Button>
    </form>
  </ContentSection>
</template>
