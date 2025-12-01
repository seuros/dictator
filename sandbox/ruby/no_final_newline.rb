class CommentsModel < ApplicationRecord
  belongs_to :post
  belongs_to :user

  validates :body, presence: true, length: { minimum: 2 }

  scope :recent, -> { order(created_at: :desc) }
  scope :by_user, ->(user_id) { where(user_id: user_id) }

  def author_name
    user.name
  end

  def post_title
    post.title
  end

  def truncated_body
    body.truncate(50)
  end
end