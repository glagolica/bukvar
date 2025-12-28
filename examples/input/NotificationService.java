package io.example.notifications;

import java.time.Instant;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * Service for sending notifications across multiple channels.
 *
 * <p>Supports email, SMS, push notifications, and webhook delivery.
 * Handles templating, delivery tracking, and retry logic.
 *
 * <h2>Usage Example</h2>
 * <pre>{@code
 * NotificationService service = new NotificationService(config);
 *
 * Notification notification = Notification.builder()
 *     .recipient("user@example.com")
 *     .channel(Channel.EMAIL)
 *     .template("welcome")
 *     .data(Map.of("name", "John"))
 *     .build();
 *
 * DeliveryResult result = service.send(notification);
 * System.out.println("Sent: " + result.getId());
 * }</pre>
 *
 * @author Platform Team
 * @version 2.5.0
 * @since 1.0.0
 * @see NotificationRepository
 * @see DeliveryTracker
 */
public class NotificationService {

    /**
     * Available notification channels.
     *
     * @since 1.0.0
     */
    public enum Channel {
        /** Email notifications via SMTP or API. */
        EMAIL,
        /** SMS text messages. */
        SMS,
        /** Mobile push notifications (iOS/Android). */
        PUSH,
        /** HTTP webhook callbacks. */
        WEBHOOK,
        /** In-app notifications. */
        IN_APP
    }

    /**
     * Priority levels for notification delivery.
     *
     * <p>Higher priority notifications are processed first and may
     * bypass rate limits in certain configurations.
     *
     * @since 2.0.0
     */
    public enum Priority {
        /** System-critical notifications (security alerts, etc.). */
        CRITICAL,
        /** Important user-facing notifications. */
        HIGH,
        /** Standard notifications (default). */
        NORMAL,
        /** Non-urgent, can be batched or delayed. */
        LOW
    }

    /**
     * Delivery status for tracking notification progress.
     */
    public enum DeliveryStatus {
        /** Notification is queued for delivery. */
        PENDING,
        /** Currently being processed. */
        PROCESSING,
        /** Successfully delivered. */
        DELIVERED,
        /** Delivery failed, may be retried. */
        FAILED,
        /** Delivery permanently failed after all retries. */
        DEAD_LETTERED,
        /** Cancelled before delivery. */
        CANCELLED
    }

    /**
     * Represents a notification to be sent.
     *
     * @param id         Unique notification identifier
     * @param recipient  Target recipient (email, phone, device token, etc.)
     * @param channel    Delivery channel
     * @param template   Template name for content generation
     * @param data       Template variables
     * @param priority   Delivery priority
     * @param scheduledAt When to send (null for immediate)
     *
     * @since 2.0.0
     */
    public record Notification(
            String id,
            String recipient,
            Channel channel,
            String template,
            Map<String, Object> data,
            Priority priority,
            Instant scheduledAt
    ) {
        /**
         * Creates a builder for constructing notifications.
         *
         * @return A new notification builder
         */
        public static Builder builder() {
            return new Builder();
        }

        /**
         * Builder for {@link Notification} instances.
         *
         * <p>Provides a fluent API for constructing notifications with
         * validation on build.
         */
        public static class Builder {
            private String recipient;
            private Channel channel;
            private String template;
            private Map<String, Object> data = Map.of();
            private Priority priority = Priority.NORMAL;
            private Instant scheduledAt;

            /**
             * Sets the notification recipient.
             *
             * @param recipient Target recipient identifier
             * @return This builder for chaining
             * @throws IllegalArgumentException if recipient is null or blank
             */
            public Builder recipient(String recipient) {
                this.recipient = recipient;
                return this;
            }

            /**
             * Sets the delivery channel.
             *
             * @param channel Notification channel
             * @return This builder for chaining
             */
            public Builder channel(Channel channel) {
                this.channel = channel;
                return this;
            }

            /**
             * Sets the template name.
             *
             * @param template Name of the template to use
             * @return This builder for chaining
             */
            public Builder template(String template) {
                this.template = template;
                return this;
            }

            /**
             * Sets template data variables.
             *
             * @param data Map of template variables
             * @return This builder for chaining
             */
            public Builder data(Map<String, Object> data) {
                this.data = data;
                return this;
            }

            /**
             * Sets the delivery priority.
             *
             * @param priority Notification priority
             * @return This builder for chaining
             */
            public Builder priority(Priority priority) {
                this.priority = priority;
                return this;
            }

            /**
             * Schedules the notification for future delivery.
             *
             * @param scheduledAt When to send the notification
             * @return This builder for chaining
             */
            public Builder scheduledAt(Instant scheduledAt) {
                this.scheduledAt = scheduledAt;
                return this;
            }

            /**
             * Builds the notification instance.
             *
             * @return A new Notification instance
             * @throws IllegalStateException if required fields are missing
             */
            public Notification build() {
                if (recipient == null || recipient.isBlank()) {
                    throw new IllegalStateException("Recipient is required");
                }
                if (channel == null) {
                    throw new IllegalStateException("Channel is required");
                }
                if (template == null || template.isBlank()) {
                    throw new IllegalStateException("Template is required");
                }
                return new Notification(
                        java.util.UUID.randomUUID().toString(),
                        recipient,
                        channel,
                        template,
                        data,
                        priority,
                        scheduledAt
                );
            }
        }
    }

    /**
     * Result of a notification delivery attempt.
     *
     * @param id            Notification ID
     * @param status        Current delivery status
     * @param deliveredAt   When delivered (if successful)
     * @param failureReason Reason for failure (if failed)
     * @param retryCount    Number of retry attempts
     */
    public record DeliveryResult(
            String id,
            DeliveryStatus status,
            Instant deliveredAt,
            String failureReason,
            int retryCount
    ) {
        /**
         * Checks if delivery was successful.
         *
         * @return {@code true} if notification was delivered
         */
        public boolean isSuccess() {
            return status == DeliveryStatus.DELIVERED;
        }

        /**
         * Checks if delivery can be retried.
         *
         * @return {@code true} if notification can be retried
         */
        public boolean isRetryable() {
            return status == DeliveryStatus.FAILED && retryCount < 3;
        }
    }

    private final ServiceConfig config;
    private final NotificationRepository repository;
    private final TemplateEngine templateEngine;

    /**
     * Creates a new notification service.
     *
     * @param config Service configuration
     * @throws IllegalArgumentException if config is null
     * @throws ServiceInitializationException if service cannot be initialized
     */
    public NotificationService(ServiceConfig config) {
        this.config = config;
        this.repository = new NotificationRepository(config);
        this.templateEngine = new TemplateEngine(config);
    }

    /**
     * Sends a notification synchronously.
     *
     * <p>Blocks until the notification is delivered or fails.
     * For non-blocking delivery, use {@link #sendAsync(Notification)}.
     *
     * @param notification The notification to send
     * @return Delivery result with status and metadata
     * @throws NotificationException if delivery fails
     * @throws InvalidRecipientException if recipient is invalid for the channel
     * @throws TemplateNotFoundException if template doesn't exist
     *
     * @see #sendAsync(Notification)
     * @see #sendBatch(List)
     */
    public DeliveryResult send(Notification notification) {
        validateNotification(notification);
        String content = templateEngine.render(notification.template(), notification.data());
        return deliverToChannel(notification, content);
    }

    /**
     * Sends a notification asynchronously.
     *
     * <p>Returns immediately with a future that completes when delivery
     * is finished or fails.
     *
     * @param notification The notification to send
     * @return CompletableFuture with delivery result
     *
     * @since 2.0.0
     */
    public CompletableFuture<DeliveryResult> sendAsync(Notification notification) {
        return CompletableFuture.supplyAsync(() -> send(notification));
    }

    /**
     * Sends multiple notifications in a batch.
     *
     * <p>Optimizes delivery by batching requests to the same channel.
     * Failed notifications are collected and returned separately.
     *
     * @param notifications List of notifications to send
     * @return Batch result with successes and failures
     *
     * @deprecated Use {@link #sendBatchAsync(List)} for better performance
     * @since 1.5.0
     */
    @Deprecated(since = "2.3.0", forRemoval = true)
    public BatchResult sendBatch(List<Notification> notifications) {
        // Implementation
        return new BatchResult(List.of(), List.of());
    }

    /**
     * Sends multiple notifications asynchronously.
     *
     * @param notifications List of notifications to send
     * @return CompletableFuture with batch result
     * @since 2.3.0
     */
    public CompletableFuture<BatchResult> sendBatchAsync(List<Notification> notifications) {
        return CompletableFuture.supplyAsync(() -> sendBatch(notifications));
    }

    /**
     * Retrieves the delivery status of a notification.
     *
     * @param notificationId The notification ID to check
     * @return Optional containing the result if found
     */
    public Optional<DeliveryResult> getStatus(String notificationId) {
        return repository.findById(notificationId);
    }

    /**
     * Cancels a pending notification.
     *
     * <p>Only notifications with status {@link DeliveryStatus#PENDING}
     * can be cancelled.
     *
     * @param notificationId The notification ID to cancel
     * @return {@code true} if cancellation was successful
     * @throws NotificationNotFoundException if notification doesn't exist
     */
    public boolean cancel(String notificationId) {
        return repository.cancel(notificationId);
    }

    /**
     * Validates a notification before sending.
     *
     * @param notification Notification to validate
     * @throws InvalidRecipientException if recipient format is invalid
     */
    private void validateNotification(Notification notification) {
        switch (notification.channel()) {
            case EMAIL -> validateEmail(notification.recipient());
            case SMS -> validatePhoneNumber(notification.recipient());
            case PUSH -> validateDeviceToken(notification.recipient());
            case WEBHOOK -> validateWebhookUrl(notification.recipient());
            case IN_APP -> validateUserId(notification.recipient());
        }
    }

    private void validateEmail(String email) {
        if (!email.contains("@")) {
            throw new InvalidRecipientException("Invalid email format: " + email);
        }
    }

    private void validatePhoneNumber(String phone) {
        if (!phone.matches("\\+?[0-9]{10,15}")) {
            throw new InvalidRecipientException("Invalid phone number: " + phone);
        }
    }

    private void validateDeviceToken(String token) {
        if (token.length() < 32) {
            throw new InvalidRecipientException("Invalid device token");
        }
    }

    private void validateWebhookUrl(String url) {
        if (!url.startsWith("https://")) {
            throw new InvalidRecipientException("Webhook must use HTTPS");
        }
    }

    private void validateUserId(String userId) {
        if (userId.isBlank()) {
            throw new InvalidRecipientException("User ID cannot be blank");
        }
    }

    private DeliveryResult deliverToChannel(Notification notification, String content) {
        // Implementation would dispatch to appropriate channel handler
        return new DeliveryResult(
                notification.id(),
                DeliveryStatus.DELIVERED,
                Instant.now(),
                null,
                0
        );
    }

    /**
     * Result of a batch notification operation.
     *
     * @param successful List of successfully delivered notifications
     * @param failed     List of failed notifications with reasons
     */
    public record BatchResult(
            List<DeliveryResult> successful,
            List<DeliveryResult> failed
    ) {
        /**
         * Gets the total number of notifications processed.
         *
         * @return Total count
         */
        public int totalCount() {
            return successful.size() + failed.size();
        }

        /**
         * Gets the success rate as a percentage.
         *
         * @return Success rate (0.0 to 100.0)
         */
        public double successRate() {
            if (totalCount() == 0) return 0.0;
            return (successful.size() * 100.0) / totalCount();
        }
    }

    // Placeholder classes for compilation
    private static class ServiceConfig {}
    private static class NotificationRepository {
        NotificationRepository(ServiceConfig config) {}
        Optional<DeliveryResult> findById(String id) { return Optional.empty(); }
        boolean cancel(String id) { return false; }
    }
    private static class TemplateEngine {
        TemplateEngine(ServiceConfig config) {}
        String render(String template, Map<String, Object> data) { return ""; }
    }
    private static class NotificationException extends RuntimeException {}
    private static class InvalidRecipientException extends RuntimeException {
        InvalidRecipientException(String msg) { super(msg); }
    }
    private static class NotificationNotFoundException extends RuntimeException {}
    private static class ServiceInitializationException extends RuntimeException {}
    private static class TemplateNotFoundException extends RuntimeException {}
}
