import { toast } from 'vue-sonner'
import { h } from 'vue'

export function showSubmittedData(
  data: unknown,
  title: string = 'You submitted the following values:'
) {
  toast.message(title, {
    description: h(
      'pre',
      { class: 'mt-2 w-full overflow-x-auto rounded-md bg-slate-950 p-4' },
      h(
        'code',
        {
          class: 'text-white',
        },
        () => JSON.stringify(data, null, 2)
      )
    ),
  })
}
