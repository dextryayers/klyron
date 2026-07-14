module Api
  class AuthController < ApplicationController
    def login
      user = User.find_by(email: params[:email])
      if user&.valid_password?(params[:password])
        token = user.generate_jwt
        render json: { token: token, user: { id: user.id, email: user.email, name: user.name } }
      else
        render json: { error: 'Invalid credentials' }, status: :unauthorized
      end
    end

    def logout
      render json: { message: 'Logged out successfully' }
    end
  end
end
