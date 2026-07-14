import Joi from 'joi'

export const idParamSchema = Joi.object({
  id: Joi.string().uuid().required(),
})

export const paginationSchema = Joi.object({
  page: Joi.number().integer().min(1).default(1),
  limit: Joi.number().integer().min(1).max(100).default(10),
})

export const createItemSchema = Joi.object({
  name: Joi.string().min(1).max(255).required(),
  description: Joi.string().max(1000).allow('').optional(),
})

export const updateItemSchema = Joi.object({
  name: Joi.string().min(1).max(255).optional(),
  description: Joi.string().max(1000).allow('').optional(),
}).min(1)
