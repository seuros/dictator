class PostsModel < ApplicationRecord
	has_many :comments
  has_many :likes

  validates :title, presence: true
	validates :body, presence: true

  scope :published, -> { where(published: true) }
  scope :recent, -> { order(created_at: :desc) }

  def public_post?
	    published && !draft?
  end

  def full_preview
    "#{title} - #{body.truncate(100)}"
  end

  protected

  def update_cache
	    Rails.cache.write("post_#{id}", self)
  end

  private

  def draft?
    !published
  end
end
