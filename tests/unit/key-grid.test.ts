import { expect, test } from 'bun:test'
import { neighbourKey } from '../../src-nuxt/app/lib/key-grid'

test('moves to the key next to it in each direction', () => {
  expect(neighbourKey(6, 'up')?.index).toBe(1)
  expect(neighbourKey(6, 'down')?.index).toBe(11)
  expect(neighbourKey(6, 'left')?.index).toBe(5)
  expect(neighbourKey(6, 'right')?.index).toBe(7)
})

test('stops at the edges of the dock without wrapping', () => {
  expect(neighbourKey(0, 'up')).toBeNull()
  expect(neighbourKey(5, 'left')).toBeNull()
  expect(neighbourKey(4, 'right')).toBeNull()
  expect(neighbourKey(11, 'down')).toBeNull()
  expect(neighbourKey(13, 'right')).toBeNull()
  expect(neighbourKey(13, 'down')).toBeNull()
})

test('the wide key is below both of the keys it spans', () => {
  expect(neighbourKey(8, 'down')).toEqual({ index: 13, column: 3 })
  expect(neighbourKey(9, 'down')).toEqual({ index: 13, column: 4 })
  expect(neighbourKey(12, 'right')?.index).toBe(13)
  expect(neighbourKey(13, 'left')?.index).toBe(12)
})

test('leaving the wide key upwards returns the way it was entered, or to its left half', () => {
  expect(neighbourKey(13, 'up', 4)?.index).toBe(9)
  expect(neighbourKey(13, 'up', 3)?.index).toBe(8)
  expect(neighbourKey(13, 'up')?.index).toBe(8)
  expect(neighbourKey(13, 'up', 0)?.index).toBe(8)
})

test('an index outside the dock goes nowhere', () => {
  expect(neighbourKey(14, 'up')).toBeNull()
})
