import { type User } from './schema'
import { faker } from '@faker-js/faker'

// Set a fixed seed for consistent data generation
faker.seed(67890)

export const users = Array.from({ length: 500 }, () => {
  const firstName = faker.person.firstName()
  const lastName = faker.person.lastName()
  return {
    id: faker.number.int(),
    name: `${firstName} ${lastName}`,
    email: faker.internet.email({ firstName }).toLocaleLowerCase(),
    notes: faker.person.jobDescriptor(),
    active: faker.helpers.arrayElement([true, false]),
    role: faker.helpers.arrayElement([2, 0]),
    last_seen: faker.date.recent(),
    created_at: faker.date.past(),
    updated_at: faker.date.recent(),
  }
}) satisfies User[]
