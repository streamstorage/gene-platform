<script setup lang="ts">
  import { roles } from '../data/data'
  import SelectDropdown from '@/components/SelectDropdown.vue'
  import { Button } from '@/components/ui/button'
  import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'
  import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
  import { Input } from '@/components/ui/input'
  import { Textarea } from '@/components/ui/textarea'
  import { showSubmittedData } from '@/lib/dev'
  import { MailPlus, Send } from '@lucide/vue'
  import { toTypedSchema } from '@vee-validate/zod'
  import { useForm } from 'vee-validate'
  import { z } from 'zod'

  const open = defineModel<boolean>()

  const formSchema = toTypedSchema(
    z.object({
      email: z.string().email('Please enter an email to invite.'),
      role: z.string().min(1, 'Role is required.'),
      desc: z.string().optional(),
    })
  )
  const form = useForm({
    validationSchema: formSchema,
  })
  const onSubmit = form.handleSubmit((values) => {
    form.resetForm()
    showSubmittedData(values)
    open.value = false
  })
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="sm:max-w-md">
      <DialogHeader class="text-start">
        <DialogTitle class="flex items-center gap-2"> <MailPlus /> Invite User </DialogTitle>
        <DialogDescription>
          Invite new user to join your team by sending them an email invitation. Assign a role to
          define their access level.
        </DialogDescription>
      </DialogHeader>
      <form
        id="user-invite-form"
        @submit="onSubmit"
        class="space-y-4"
      >
        <FormField
          name="email"
          v-slot="{ componentField }"
        >
          <FormItem>
            <FormLabel>Email</FormLabel>
            <FormControl>
              <Input
                type="email"
                placeholder="eg: john.doe@gmail.com"
                v-bind="componentField"
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
        <FormField
          name="role"
          v-slot="{ componentField }"
        >
          <FormItem>
            <FormLabel>Role</FormLabel>
            <FormControl>
              <SelectDropdown
                v-bind="componentField"
                placeholder="Select a role"
                :items="
                  roles.map(({ label, value }) => ({
                    label,
                    value,
                  }))
                "
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
        <FormField
          name="desc"
          v-slot="{ componentField }"
        >
          <FormItem>
            <FormLabel>Description (optional)</FormLabel>
            <FormControl>
              <Textarea
                class="resize-none"
                placeholder="Add a personal note to your invitation (optional)"
                v-bind="componentField"
              />
            </FormControl>
            <FormMessage />
          </FormItem>
        </FormField>
      </form>
      <DialogFooter class="gap-y-2">
        <DialogClose as-child>
          <Button variant="outline">Cancel</Button>
        </DialogClose>
        <Button
          type="submit"
          form="user-invite-form"
        >
          Invite <Send />
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
