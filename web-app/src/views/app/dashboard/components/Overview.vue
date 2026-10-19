<script setup lang="ts">
  import { Bar, BarChart, XAxis, YAxis } from 'vccs'

  const data = [
    {
      name: 'Jan',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Feb',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Mar',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Apr',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'May',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Jun',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Jul',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Aug',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Sep',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Oct',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Nov',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
    {
      name: 'Dec',
      total: Math.floor(Math.random() * 5000) + 1000,
    },
  ]
</script>

<template>
  <BarChart
    :data="data"
    :height="350"
  >
    <XAxis
      data-key="name"
      stroke="#888888"
      :font-size="12"
      :tick-line="false"
      :axis-line="false"
    />
    <YAxis
      direction="ltr"
      stroke="#888888"
      :font-size="12"
      :tick-line="false"
      :axis-line="false"
      :tick-formatter="(value) => `$${value}`"
    />
    <Bar
      data-key="total"
      fill="currentColor"
      :radius="[4, 4, 0, 0]"
      class="fill-primary"
    />
  </BarChart>
</template>
