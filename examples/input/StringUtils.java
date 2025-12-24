package com.example;

/**
 * Sample Java class demonstrating JavaDoc documentation.
 * 
 * <p>This class provides utility methods for string manipulation
 * and demonstrates various JavaDoc tags.</p>
 * 
 * @author Glagolica
 * @version 1.0.0
 * @since 1.0
 * @see StringBuilder
 */
public class StringUtils {

    /**
     * Reverses the given string.
     * 
     * <p>This method creates a new string with characters
     * in reverse order.</p>
     * 
     * @param input the string to reverse, must not be null
     * @return the reversed string
     * @throws NullPointerException if input is null
     * @since 1.0
     * 
     * @example
     * <pre>{@code
     * String result = StringUtils.reverse("hello");
     * // result = "olleh"
     * }</pre>
     */
    public static String reverse(String input) {
        if (input == null) {
            throw new NullPointerException("Input cannot be null");
        }
        return new StringBuilder(input).reverse().toString();
    }

    /**
     * Checks if a string is empty or null.
     * 
     * @param str the string to check
     * @return {@code true} if the string is null or empty,
     *         {@code false} otherwise
     * @deprecated Use {@link #isBlank(String)} instead for
     *             whitespace handling.
     */
    @Deprecated
    public static boolean isEmpty(String str) {
        return str == null || str.isEmpty();
    }

    /**
     * Checks if a string is blank (null, empty, or whitespace only).
     * 
     * @param str the string to check
     * @return true if blank, false otherwise
     */
    public static boolean isBlank(String str) {
        return str == null || str.trim().isEmpty();
    }
}
