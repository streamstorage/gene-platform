<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
  } from '@/components/ui/card'
  import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
  import { Input } from '@/components/ui/input'
  import PasswordInput from '@/components/password-input.vue'
  import { Loader2, LogIn } from '@lucide/vue'
  import { toTypedSchema } from '@vee-validate/zod'
  import { useForm } from 'vee-validate'
  import { ref } from 'vue'
  import { z } from 'zod'

  const isLoading = ref(false)

  const formSchema = toTypedSchema(
    z.object({
      email: z.string().email('Please enter your email.'),
      password: z.string().min(1, 'Please enter your password.'),
    })
  )
  const form = useForm({
    validationSchema: formSchema,
  })
  const onSubmit = form.handleSubmit((values) => {
    console.log('Form submitted!', values)
  })
</script>

<template>
  <Card class="max-w-sm gap-4">
    <CardHeader>
      <CardTitle class="text-lg tracking-tight">Sign in</CardTitle>
      <CardDescription>
        Enter your email and password below to log into{{ ' ' }} <br class="max-sm:hidden" />
        your account.
      </CardDescription>
    </CardHeader>
    <CardContent>
      <form
        @submit="onSubmit"
        class="grid gap-3"
      >
        <FormField
          v-slot="{ componentField }"
          name="email"
        >
          <FormItem>
            <FormLabel>Email</FormLabel>
            <FormControl>
              <Input
                placeholder="name@example.com"
                v-bind="componentField"
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
        <FormField
          v-slot="{ componentField }"
          name="password"
        >
          <FormItem class="relative">
            <FormLabel for="auth-password-id">Password</FormLabel>
            <FormControl>
              <PasswordInput
                placeholder="********"
                v-bind="componentField"
              />
            </FormControl>
            <FormMessage />
            <RouterLink
              to="/forgot-password"
              class="absolute inset-e-0 -top-0.5 text-sm font-medium text-muted-foreground hover:opacity-75"
            >
              Forgot password?
            </RouterLink>
          </FormItem>
        </FormField>
        <Button
          class="mt-2"
          :disabled="isLoading"
        >
          <Loader2
            v-if="isLoading"
            class="animate-spin"
          />
          <LogIn v-if="!isLoading" />
          Sign in
        </Button>
      </form>
    </CardContent>

    <CardFooter>
      <p class="px-8 text-center text-sm text-muted-foreground">
        By clicking sign in, you agree to our{{ ' ' }}
        <a
          href="/terms"
          class="underline underline-offset-4 hover:text-primary"
        >
          Terms of Service </a
        >{{ ' ' }} and{{ ' ' }}
        <a
          href="/privacy"
          class="underline underline-offset-4 hover:text-primary"
        >
          Privacy Policy
        </a>
        .
      </p>
    </CardFooter>
  </Card>
</template>
