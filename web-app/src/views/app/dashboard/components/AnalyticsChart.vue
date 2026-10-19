<script setup lang="ts">
  import { Area, AreaChart, XAxis, YAxis } from 'vccs'

  const data = [
    {
      name: 'Mon',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
    {
      name: 'Tue',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
    {
      name: 'Wed',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
    {
      name: 'Thu',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
    {
      name: 'Fri',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
    {
      name: 'Sat',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
    {
      name: 'Sun',
      clicks: Math.floor(Math.random() * 900) + 100,
      uniques: Math.floor(Math.random() * 700) + 80,
    },
  ]
</script>

<template>
  <AreaChart
    :data="data"
    :height="300"
  >
    <XAxis
      data-key="name"
      stroke="#888888"
      :font-size="12"
      :tick-line="false"
      :axis-line="false"
    />
    <YAxis
      stroke="#888888"
      :font-size="12"
      :tick-line="false"
      :axis-line="false"
    />
    <Area
      type="monotone"
      data-key="clicks"
      stroke="currentColor"
      class="text-primary"
      fill="currentColor"
      :fill-opacity="0.15"
    />
    <Area
      type="monotone"
      data-key="uniques"
      stroke="currentColor"
      class="text-muted-foreground"
      fill="currentColor"
      :fill-opacity="0.1"
    />
  </AreaChart>
</template>
