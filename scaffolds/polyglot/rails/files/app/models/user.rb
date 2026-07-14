class User < ApplicationRecord
  devise :database_authenticatable, :registerable,
         :recoverable, :rememberable, :validatable,
         :jwt_authenticatable, jwt_revocation_strategy: self

  validates :name, presence: true
  validates :email, presence: true, uniqueness: true

  def self.jwt_revoked?(payload, user)
    false
  end

  def self.revoke_jwt(payload, user)
  end

  def generate_jwt
    payload = { sub: id, exp: 24.hours.from_now.to_i }
    JWT.encode(payload, ENV['DEVISE_JWT_SECRET'], 'HS256')
  end
end
