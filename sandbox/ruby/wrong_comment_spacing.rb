class OrdersController < ApplicationController
  #Authenticate user before proceeding
  before_action :authenticate_user!
  #Check admin status for admin actions
  before_action :check_admin, only: [:destroy, :bulk_update]

  def index
    #Fetch all orders for current user
    @orders = current_user.orders.includes(:items)
    #Sort by created date descending
    @orders = @orders.order(created_at: :desc)
    render :index
  end

  def show
    #Find order by id
    @order = Order.find(params[:id])
    #Check authorization
    authorize_user(@order)
    render :show
  end

  def create
    #Build new order from params
    @order = current_user.orders.build(order_params)
    #Save and render response
    if @order.save
      render :show, status: :created
    else
      render :errors, status: :unprocessable_entity
    end
  end

  private

  def order_params
    #Permit order attributes
    params.require(:order).permit(:status, :notes, items_attributes: [:id, :quantity])
  end

  def check_admin
    #Redirect if not admin
    redirect_to root_path unless current_user.admin?
  end
end
