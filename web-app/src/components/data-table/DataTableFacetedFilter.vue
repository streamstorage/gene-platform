<script setup lang="ts" generic="T extends RowData">
  import type { DataTableColumn, FacetedFilterOption } from '.'
  import { cn } from '@/lib/utils'
  import { Badge } from '@/components/ui/badge'
  import { Button } from '@/components/ui/button'
  import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
    CommandSeparator,
  } from '@/components/ui/command'
  import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
  import { Separator } from '@/components/ui/separator'
  import { CheckIcon, CirclePlusIcon } from '@lucide/vue'
  import type { RowData } from '@tanstack/vue-table'
  import { computed } from 'vue'

  interface DataTableFacetedFilterProps {
    column?: DataTableColumn<T>
    title?: string
    options: FacetedFilterOption[]
  }

  const props = defineProps<DataTableFacetedFilterProps>()

  const selectedValues = computed(() => {
    const values = props.column?.getFilterValue()
    if (Array.isArray(values)) {
      return new Set(values as unknown[])
    } else if (values) {
      return new Set([values] as unknown[])
    } else {
      return new Set([] as unknown[])
    }
  })

  const facets = computed(() => props.column?.getFacetedUniqueValues())

  function handleSelect(option: FacetedFilterOption) {
    const isSelected = selectedValues.value.has(option.value)
    if (isSelected) {
      selectedValues.value.delete(option.value)
    } else {
      selectedValues.value.add(option.value)
    }
    const filterValues = Array.from(selectedValues.value)
    props.column?.setFilterValue(filterValues.length ? filterValues : undefined)
  }
</script>

<template>
  <Popover>
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        size="sm"
        class="h-8 border-dashed"
      >
        <CirclePlusIcon class="size-4" />
        {{ title }}
        <template v-if="selectedValues.size > 0">
          <Separator
            orientation="vertical"
            class="mx-2 h-4"
          />
          <Badge
            variant="secondary"
            class="rounded-sm px-1 font-normal lg:hidden"
          >
            {{ selectedValues.size }}
          </Badge>
          <div class="hidden space-x-1 lg:flex">
            <Badge
              v-if="selectedValues.size > 2"
              variant="secondary"
              class="rounded-sm px-1 font-normal"
            >
              {{ selectedValues.size }} selected
            </Badge>
            <template v-else>
              <Badge
                v-for="option in props.options.filter((option) => selectedValues.has(option.value))"
                :key="option.value as string"
                variant="secondary"
                class="rounded-sm px-1 font-normal"
              >
                {{ option.label }}
              </Badge>
            </template>
          </div>
        </template>
      </Button>
    </PopoverTrigger>
    <PopoverContent
      class="w-50 p-0"
      align="start"
    >
      <Command>
        <CommandInput :placeholder="props.title" />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup>
            <CommandItem
              v-for="option in options"
              :key="option.value as string"
              :value="option"
              @select="handleSelect(option)"
            >
              <div
                :class="
                  cn(
                    'flex size-4 items-center justify-center rounded-sm border border-primary',
                    selectedValues.has(option.value)
                      ? 'bg-primary text-primary-foreground'
                      : 'opacity-50 [&_svg]:invisible'
                  )
                "
              >
                <CheckIcon class="h-4 w-4 text-background" />
              </div>
              <component
                :is="option.icon"
                v-if="option.icon"
                class="size-4 text-muted-foreground"
              />
              <span>{{ option.label }}</span>
              <span
                v-if="facets?.get(option.value)"
                class="ms-auto flex h-4 w-4 items-center justify-center font-mono text-xs"
              >
                {{ facets.get(option.value) }}
              </span>
            </CommandItem>
          </CommandGroup>

          <template v-if="selectedValues.size > 0">
            <CommandSeparator />
            <CommandGroup>
              <CommandItem
                :value="{ label: 'Clear filters' }"
                @select="column?.setFilterValue(undefined)"
                class="justify-center text-center"
              >
                Clear filters
              </CommandItem>
            </CommandGroup>
          </template>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
