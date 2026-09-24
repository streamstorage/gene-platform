import '@tanstack/vue-table'

declare module '@tanstack/vue-table' {
  interface ColumnMeta<TData, TValue> {
    class?: string // apply to both th and td
    tdClass?: string
    thClass?: string
  }
}
