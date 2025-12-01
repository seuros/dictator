class UsersController < ApplicationController  
  before_action :authenticate_user!

  def index    
    @users = User.all
    render :index
  end

  def show
    @user = User.find(params[:id])
    if @user.present?
      render :show
    else
      render :not_found, status: 404
    end
  end

  private

  def user_params
    params.require(:user).permit(:name, :email)
  end
end
