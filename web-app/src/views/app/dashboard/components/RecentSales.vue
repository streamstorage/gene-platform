<script setup lang="ts">
  import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
</script>

<template>
  <div class="space-y-8">
    <div class="flex items-center gap-4">
      <Avatar class="h-9 w-9">
        <AvatarImage
          src="https://images.unsplash.com/profile-1630014874714-0735d09c8bb8image?w=40&dpr=2&crop=faces&bg=%23fff&h=40&auto=format&fit=crop&q=60&ixlib=rb-4.1.0"
          alt="Avatar"
        />
        <AvatarFallback>OM</AvatarFallback>
      </Avatar>
      <div class="flex flex-1 flex-wrap items-center justify-between">
        <div class="space-y-1">
          <p class="text-sm leading-none font-medium">Olivia Martin</p>
          <p class="text-sm text-muted-foreground">olivia.martin@email.com</p>
        </div>
        <div class="font-medium">+$1,999.00</div>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <Avatar class="flex h-9 w-9 items-center justify-center space-y-0 border">
        <AvatarImage
          src="https://images.unsplash.com/profile-1779230097291-972d27db5e5cimage?w=40&dpr=2&crop=faces&bg=%23fff&h=40&auto=format&fit=crop&q=60&ixlib=rb-4.1.0"
          alt="Avatar"
        />
        <AvatarFallback>JL</AvatarFallback>
      </Avatar>
      <div class="flex flex-1 flex-wrap items-center justify-between">
        <div class="space-y-1">
          <p class="text-sm leading-none font-medium">Jackson Lee</p>
          <p class="text-sm text-muted-foreground">jackson.lee@email.com</p>
        </div>
        <div class="font-medium">+$39.00</div>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <Avatar class="h-9 w-9">
        <AvatarImage
          src="https://images.unsplash.com/profile-1764137444759-0c2c046e77e4?w=40&dpr=1&crop=faces&bg=%23fff&h=40&auto=format&fit=crop&q=60&ixlib=rb-4.1.0"
          alt="Avatar"
        />
        <AvatarFallback>IN</AvatarFallback>
      </Avatar>
      <div class="flex flex-1 flex-wrap items-center justify-between">
        <div class="space-y-1">
          <p class="text-sm leading-none font-medium">Isabella Nguyen</p>
          <p class="text-sm text-muted-foreground">isabella.nguyen@email.com</p>
        </div>
        <div class="font-medium">+$299.00</div>
      </div>
    </div>

    <div class="flex items-center gap-4">
      <Avatar class="h-9 w-9">
        <AvatarImage
          src=""
          alt="Avatar"
        />
        <AvatarFallback>WK</AvatarFallback>
      </Avatar>
      <div class="flex flex-1 flex-wrap items-center justify-between">
        <div class="space-y-1">
          <p class="text-sm leading-none font-medium">William Kim</p>
          <p class="text-sm text-muted-foreground">will@email.com</p>
        </div>
        <div class="font-medium">+$99.00</div>
      </div>
    </div>

    <div class="flex items-center gap-4">
      <Avatar class="h-9 w-9">
        <AvatarImage
          src=""
          alt="Avatar"
        />
        <AvatarFallback>SD</AvatarFallback>
      </Avatar>
      <div class="flex flex-1 flex-wrap items-center justify-between">
        <div class="space-y-1">
          <p class="text-sm leading-none font-medium">Sofia Davis</p>
          <p class="text-sm text-muted-foreground">sofia.davis@email.com</p>
        </div>
        <div class="font-medium">+$39.00</div>
      </div>
    </div>
  </div>
</template>
