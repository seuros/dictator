class ProductsController < ApplicationController
  before_action :authenticate_user!
  before_action :load_product, only: [:show, :edit, :update, :destroy]

  def index
    @products = Product.all
    @products = @products.where(category: params[:category]) if params[:category].present?
    @products = @products.where('name LIKE ?', "%#{params[:search]}%") if params[:search].present?
    @products = @products.page(params[:page]).per(20)
    render :index
  end

  def show
    render :show
  end

  def new
    @product = Product.new
    render :new
  end

  def create
    @product = Product.new(product_params)
    if @product.save
      redirect_to @product, notice: 'Product created successfully'
    else
      render :new, status: :unprocessable_entity
    end
  end

  def edit
    render :edit
  end

  def update
    if @product.update(product_params)
      redirect_to @product, notice: 'Product updated successfully'
    else
      render :edit, status: :unprocessable_entity
    end
  end

  def destroy
    @product.destroy
    redirect_to products_url, notice: 'Product destroyed successfully'
  end

  private

  def load_product
    @product = Product.find(params[:id])
  end

  def product_params
    params.require(:product).permit(:name, :description, :price, :category, :stock)
  end

  # Utility method 1
  def process_bulk_upload(file)
    results = []
    CSV.foreach(file.path, headers: true) do |row|
      product = Product.new(row.to_h)
      results << { success: product.save, product: product }
    end
    results
  end

  # Utility method 2
  def export_products(products)
    CSV.generate do |csv|
      csv << ['ID', 'Name', 'Description', 'Price', 'Category', 'Stock']
      products.each do |product|
        csv << [product.id, product.name, product.description, product.price, product.category, product.stock]
      end
    end
  end

  # Utility method 3
  def recalculate_inventory
    Product.find_each do |product|
      product.update(stock: product.order_items.sum(:quantity))
    end
  end

  # Utility method 4
  def sync_with_external_api
    external_products = ExternalAPI.fetch_products
    external_products.each do |ext_product|
      product = Product.find_or_initialize_by(external_id: ext_product['id'])
      product.update(
        name: ext_product['name'],
        description: ext_product['description'],
        price: ext_product['price']
      )
    end
  end

  # Utility method 5
  def generate_product_report
    {
      total_products: Product.count,
      total_value: Product.sum(:price),
      by_category: Product.group(:category).count,
      low_stock: Product.where('stock < ?', 10).count
    }
  end

  # Utility method 6
  def apply_discount(discount_percentage)
    Product.update_all("price = price * #{100 - discount_percentage} / 100.0")
  end

  # Utility method 7
  def archive_old_products(months_ago = 12)
    cutoff_date = months_ago.months.ago
    archived_count = Product.where('updated_at < ?', cutoff_date).update_all(archived: true)
    archived_count
  end

  # Utility method 8
  def get_trending_products
    Product.joins(:order_items)
      .select('products.*, COUNT(order_items.id) as order_count')
      .group('products.id')
      .order('order_count DESC')
      .limit(10)
  end

  # Utility method 9
  def get_related_products(product, limit = 5)
    Product.where(category: product.category)
      .where.not(id: product.id)
      .limit(limit)
  end

  # Utility method 10
  def validate_product_data(product_data)
    errors = []
    errors << 'Name is required' if product_data[:name].blank?
    errors << 'Price must be positive' if product_data[:price].to_f <= 0
    errors << 'Description is too short' if product_data[:description].to_s.length < 10
    errors << 'Category is invalid' unless valid_categories.include?(product_data[:category])
    errors
  end

  # Utility method 11
  def valid_categories
    ['Electronics', 'Clothing', 'Books', 'Home', 'Sports', 'Toys', 'Food']
  end

  # Utility method 12
  def generate_product_slug(name)
    name.downcase.tr(' ', '-').gsub(/[^a-z0-9-]/, '')
  end

  # Utility method 13
  def calculate_average_rating
    Product.find_each do |product|
      avg_rating = product.reviews.average(:rating)
      product.update(average_rating: avg_rating)
    end
  end

  # Utility method 14
  def send_product_notifications
    new_products = Product.where('created_at > ?', 1.day.ago)
    User.where(notifications_enabled: true).each do |user|
      ProductNotificationService.notify(user, new_products)
    end
  end

  # Utility method 15
  def cleanup_draft_products
    Product.where(draft: true).where('updated_at < ?', 7.days.ago).delete_all
  end

  # Utility method 16
  def generate_product_summary(product)
    "#{product.name} - #{product.category} - $#{product.price}"
  end

  # Utility method 17
  def validate_bulk_update(products)
    products.each do |product|
      return false unless product.valid?
    end
    true
  end

  # Utility method 18
  def bulk_email_notification(products)
    products.each do |product|
      ProductMailer.new_product(product).deliver_later
    end
  end

  # Utility method 19
  def calculate_stock_value
    Product.sum('price * stock')
  end

  # Utility method 20
  def get_products_by_price_range(min_price, max_price)
    Product.where('price >= ? AND price <= ?', min_price, max_price)
  end

  # Utility method 21
  def mark_products_as_featured(product_ids)
    Product.where(id: product_ids).update_all(featured: true)
  end

  # Utility method 22
  def unmark_featured_products(product_ids)
    Product.where(id: product_ids).update_all(featured: false)
  end

  # Utility method 23
  def get_featured_products
    Product.where(featured: true).order(created_at: :desc)
  end

  # Utility method 24
  def duplicate_product(product_id)
    original = Product.find(product_id)
    duplicate = original.dup
    duplicate.name = "#{original.name} (Copy)"
    duplicate.save
    duplicate
  end

  # Utility method 25
  def bulk_price_update(product_ids, new_price)
    Product.where(id: product_ids).update_all(price: new_price)
  end

  # Utility method 26
  def get_low_stock_products(threshold = 10)
    Product.where('stock < ?', threshold)
  end

  # Utility method 27
  def restock_notification(product)
    RestockMailer.notify(product).deliver_later
  end

  # Utility method 28
  def archive_category(category_name)
    Product.where(category: category_name).update_all(archived: true)
  end

  # Utility method 29
  def unarchive_category(category_name)
    Product.where(category: category_name).update_all(archived: false)
  end

  # Utility method 30
  def get_archived_products
    Product.where(archived: true)
  end

  # Utility method 31
  def calculate_total_inventory_cost
    Product.sum('price * stock')
  end

  # Utility method 32
  def get_products_updated_since(date)
    Product.where('updated_at > ?', date)
  end

  # Utility method 33
  def get_products_created_since(date)
    Product.where('created_at > ?', date)
  end

  # Utility method 34
  def bulk_add_tag(product_ids, tag)
    Product.where(id: product_ids).each do |product|
      product.tags << tag unless product.tags.include?(tag)
    end
  end

  # Utility method 35
  def bulk_remove_tag(product_ids, tag)
    Product.where(id: product_ids).each do |product|
      product.tags.delete(tag)
    end
  end

  # Utility method 36
  def get_products_by_tag(tag)
    Product.joins(:tags).where(tags: { name: tag })
  end

  # Utility method 37
  def get_all_tags
    Tag.distinct.pluck(:name)
  end

  # Utility method 38
  def merge_duplicate_products(product_ids)
    original = Product.find(product_ids.first)
    duplicates = Product.where(id: product_ids.drop(1))
    duplicates.update_all(merged_into: original.id)
  end

  # Utility method 39
  def restore_merged_products(product_id)
    Product.where(merged_into: product_id).update_all(merged_into: nil)
  end

  # Utility method 40
  def get_merge_history(product_id)
    Product.where(merged_into: product_id)
  end
end
