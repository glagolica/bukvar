//! Documentation comment parsers for JSDoc, JavaDoc, and PyDoc

pub mod javadoc;
pub mod jsdoc;
pub mod pydoc;

pub use javadoc::JavaDocParser;
pub use jsdoc::JsDocParser;
pub use pydoc::PyDocParser;

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::{DocumentType, NodeKind};

  #[test]
  fn test_jsdoc_basic() {
    let input = r#"
/**
 * This is a description
 * @param {string} name - The name
 * @returns {void}
 */
function test() {}
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::JavaScript);
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_multiple_comments() {
    let input = r#"
/** First comment */
function first() {}

/** Second comment */
function second() {}
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 2);
  }

  #[test]
  fn test_jsdoc_empty() {
    let input = "function test() {}";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_skip_normal_comments() {
    let input = r#"
/* This is not a JSDoc comment */
// Neither is this
/** But this is */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 1);
  }

  #[test]
  fn test_javadoc_basic() {
    let input = r#"
/**
 * This is a description
 * @param name The name parameter
 * @return The result
 */
public void test() {}
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::Java);
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_with_throws() {
    let input = r#"
/**
 * Description
 * @param x Input value
 * @throws IllegalArgumentException if x is negative
 * @return Result
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_empty() {
    let input = "public class Test {}";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_google_style() {
    let input = r#"
def test():
    """This is a description.

    Args:
        name: The name parameter
        value: The value

    Returns:
        The result
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.doc_type, DocumentType::Python);
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_numpy_style() {
    let input = r#"
def test():
    """
    This is a description.

    Parameters
    ----------
    name : str
        The name parameter

    Returns
    -------
    str
        The result
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_empty() {
    let input = "def test(): pass";
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_single_line() {
    let input = r#"
def test():
    """Single line docstring."""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_parsers_doc_comment_node() {
    let input = "/** Test */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    if !doc.nodes.is_empty() {
      assert!(matches!(doc.nodes[0].kind, NodeKind::DocComment { .. }));
    }
  }

  // ============================================
  // JSDOC EDGE CASES
  // ============================================

  #[test]
  fn test_jsdoc_unicode_content() {
    let input = r#"
/**
 * Функция для обработки данных 🎉
 * @param {string} имя - 名前
 * @returns {void}
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_triple_star() {
    let input = "/*** This has three stars, not JSDoc */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(
      doc.nodes.is_empty(),
      "Triple star should not be parsed as JSDoc"
    );
  }

  #[test]
  fn test_jsdoc_minimal() {
    let input = "/** */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_single_line() {
    let input = "/** Single line comment */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_no_leading_stars() {
    let input = r#"/**
Description without leading stars
@param x The x value
*/"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_complex_types() {
    let input = r#"
/**
 * @param {Array<{name: string, value: number}>} items - Complex type
 * @param {Map<string, Set<number>>} map - Nested generics
 * @param {(a: number, b: string) => boolean} callback - Function type
 * @returns {Promise<{result: T, error: Error | null}>}
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_typedef() {
    let input = r#"
/**
 * @typedef {Object} Person
 * @property {string} name - The name
 * @property {number} age - The age
 * @property {Address} [address] - Optional address
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_callback() {
    let input = r#"
/**
 * @callback CompareFunction
 * @param {*} a - First value
 * @param {*} b - Second value
 * @returns {number}
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_template() {
    let input = r#"
/**
 * @template T
 * @template {string} K
 * @param {T} value
 * @returns {K}
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_all_tags() {
    let input = r#"
/**
 * Main description
 * @module myModule
 * @version 1.0.0
 * @author John Doe <john@example.com>
 * @since 0.1.0
 * @deprecated Use newFunction instead
 * @see OtherClass
 * @example
 * const result = myFunc();
 * @param {string} name - Parameter description
 * @param {number} [count=0] - Optional with default
 * @returns {boolean} Return description
 * @throws {Error} When something fails
 * @fires change
 * @listens click
 * @async
 * @generator
 * @override
 * @abstract
 * @access private
 * @private
 * @public
 * @protected
 * @readonly
 * @static
 * @this {MyClass}
 * @ignore
 * @todo Implement this
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_multiline_description() {
    let input = r#"
/**
 * This is the first line of the description.
 * This is the second line.
 *
 * This is after a blank line, still description.
 *
 * @param x Value
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_inline_tags() {
    let input = r#"
/**
 * See {@link OtherClass} for more info.
 * Also check {@linkcode someFunction} and {@linkplain plainLink}.
 * Tutorial: {@tutorial getting-started}
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_example_with_code() {
    let input = r#"
/**
 * @example
 * // This is a comment
 * const x = myFunc({
 *   name: "test",
 *   value: 42
 * });
 *
 * @example <caption>Named example</caption>
 * myFunc("simple");
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_destructured_param() {
    let input = r#"
/**
 * @param {Object} options
 * @param {string} options.name - The name
 * @param {number} options.count - The count
 * @param {Object} options.nested
 * @param {string} options.nested.value - Nested value
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_nullable_types() {
    let input = r#"
/**
 * @param {?string} nullable - Nullable string
 * @param {!string} nonNull - Non-null string
 * @param {string=} optional - Optional param
 * @param {...string} rest - Rest params
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_special_chars_in_description() {
    let input = r#"
/**
 * Handles < and > symbols, also & and "quotes".
 * Code: `x < y && y > z`
 * @param {string} html - Contains <tags> and &entities;
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_empty_param_description() {
    let input = r#"
/**
 * @param {string} name
 * @param {number} value
 * @returns {void}
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_unclosed_brace() {
    let input = r#"
/**
 * @param {Object options - Missing closing brace
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    // Should handle gracefully
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_class_and_constructor() {
    let input = r#"
/**
 * @class
 * @classdesc A person class
 * @constructor
 * @param {string} name
 * @extends BaseClass
 * @implements {Serializable}
 * @mixes EventEmitter
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_at_symbol_in_email() {
    let input = r#"
/**
 * Contact: user@example.com
 * @author Team <team@company.org>
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_jsdoc_consecutive_comments() {
    let input = r#"
/** First */
/** Second */
/** Third */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 3);
  }

  #[test]
  fn test_jsdoc_with_markdown() {
    let input = r#"
/**
 * # Heading
 *
 * - List item 1
 * - List item 2
 *
 * ```js
 * const x = 1;
 * ```
 *
 * **Bold** and *italic*
 */
"#;
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // JAVADOC EDGE CASES
  // ============================================

  #[test]
  fn test_javadoc_unicode_content() {
    let input = r#"
/**
 * Обработка данных 日本語 🎉
 * @param имя параметр
 * @return 結果
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_triple_star() {
    let input = "/*** Three stars */";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_minimal() {
    let input = "/** */";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_single_line() {
    let input = "/** Single line */";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_all_tags() {
    let input = r#"
/**
 * Main description
 *
 * @param name The name parameter
 * @param <T> Type parameter
 * @return The result
 * @throws IllegalArgumentException if invalid
 * @throws NullPointerException if null
 * @exception RuntimeException on error
 * @see OtherClass
 * @see OtherClass#method()
 * @see <a href="http://example.com">Link</a>
 * @since 1.0
 * @version 2.0
 * @author John Doe
 * @deprecated Use newMethod instead
 * @serial field description
 * @serialField name type description
 * @serialData data description
 * @inheritDoc
 * @hidden
 * @apiNote API usage note
 * @implSpec Implementation specification
 * @implNote Implementation note
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_code_and_literal() {
    let input = r#"
/**
 * Use {@code null} for empty value.
 * The method {@literal <T>} handles generics.
 * Example: {@code Map<String, List<Integer>>}
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_link_variants() {
    let input = r#"
/**
 * See {@link Object}
 * See {@link Object#equals(Object)}
 * See {@link #localMethod()}
 * See {@linkplain String plain link}
 * Value is {@value #CONSTANT}
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_html_content() {
    let input = r#"
/**
 * <p>This is a paragraph.</p>
 * <ul>
 *   <li>Item 1</li>
 *   <li>Item 2</li>
 * </ul>
 * <pre>{@code
 * String s = "example";
 * }</pre>
 * <table>
 *   <tr><td>Cell</td></tr>
 * </table>
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_generics() {
    let input = r#"
/**
 * @param <K> The key type
 * @param <V> The value type
 * @param map The map parameter of type {@code Map<K, V>}
 * @return A {@code List<Map.Entry<K, V>>}
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_multiline_param() {
    let input = r#"
/**
 * @param name This is a very long description
 *             that spans multiple lines because
 *             we need to explain things in detail
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_multiple_throws() {
    let input = r#"
/**
 * @throws IOException when IO fails
 * @throws SQLException when database fails
 * @throws IllegalStateException when state is invalid
 * @throws NullPointerException when argument is null
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_record() {
    let input = r#"
/**
 * A point record.
 * @param x the x coordinate
 * @param y the y coordinate
 */
public record Point(int x, int y) {}
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_snippet() {
    let input = r#"
/**
 * Example:
 * {@snippet :
 * public static void main(String[] args) {
 *     System.out.println("Hello");
 * }
 * }
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_consecutive() {
    let input = r#"
/** First comment */
/** Second comment */
/** Third comment */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 3);
  }

  #[test]
  fn test_javadoc_special_chars() {
    let input = r#"
/**
 * Handles &lt; and &gt; symbols
 * Also &amp; and &quot;
 * @param value contains < > & "
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_javadoc_empty_tags() {
    let input = r#"
/**
 * @param
 * @return
 * @throws
 */
"#;
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    // Should handle empty tags gracefully
    assert!(!doc.nodes.is_empty());
  }

  // ============================================
  // PYDOC EDGE CASES
  // ============================================

  #[test]
  fn test_pydoc_unicode_content() {
    let input = r#"
def test():
    """
    Обработка данных 日本語 🎉

    Args:
        имя: параметр
        名前: パラメータ

    Returns:
        結果
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_single_quotes() {
    let input = r#"
def test():
    '''Single quoted docstring.'''
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_mixed_quotes() {
    let input = r#"
def test():
    """
    Contains 'single quotes' and "double quotes"
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_google_full() {
    let input = r#"
def complex_func():
    """Short summary line.

    Extended description that spans
    multiple lines and paragraphs.

    Second paragraph of description.

    Args:
        param1 (int): First parameter.
        param2 (str): Second parameter with a long
            description that wraps.
        *args: Variable positional arguments.
        **kwargs: Variable keyword arguments.

    Returns:
        dict: A dictionary containing:
            - key1: Description of key1
            - key2: Description of key2

    Yields:
        int: Each number in sequence.

    Raises:
        ValueError: If param1 is negative.
        TypeError: If param2 is not a string.

    Examples:
        >>> result = complex_func()
        >>> print(result)
        {'key1': 1}

        Multiple examples:

        >>> complex_func(1, 'a')
        >>> complex_func(2, 'b')

    Note:
        This is an important note.

    Warning:
        This is a warning message.

    See Also:
        other_func: Related function.

    References:
        [1] Some reference
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_numpy_full() {
    let input = r#"
def numpy_func():
    """
    Short summary.

    Extended description.

    Parameters
    ----------
    param1 : int
        Description of param1.
    param2 : str, optional
        Description of param2 (default is 'hello').
    *args : tuple
        Variable arguments.
    **kwargs : dict
        Keyword arguments.

    Returns
    -------
    ndarray
        Description of return value.
    int
        Second return value.

    Yields
    ------
    int
        Yielded values.

    Raises
    ------
    ValueError
        If something is wrong.

    Warns
    -----
    UserWarning
        When something is deprecated.

    Other Parameters
    ----------------
    extra : bool
        An extra parameter.

    Attributes
    ----------
    attr1 : int
        First attribute.

    See Also
    --------
    related_func : Description.

    Notes
    -----
    These are some notes.

    References
    ----------
    .. [1] Reference one
    .. [2] Reference two

    Examples
    --------
    >>> result = numpy_func()
    >>> print(result)
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_sphinx_full() {
    let input = r#"
def sphinx_func():
    """Short description.

    :param param1: First parameter
    :type param1: int
    :param param2: Second parameter
    :type param2: str
    :keyword key1: Keyword arg
    :raises ValueError: When invalid
    :raises TypeError: When wrong type
    :returns: The result
    :rtype: dict
    :var myvar: A variable
    :vartype myvar: list
    :ivar instance_var: Instance variable
    :cvar class_var: Class variable

    .. note::
       This is a note.

    .. warning::
       This is a warning.

    .. seealso::
       :func:`other_func`

    .. versionadded:: 1.0
    .. versionchanged:: 2.0
    .. deprecated:: 3.0
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_empty_docstring() {
    let input = r#"
def test():
    """"""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    // Should handle empty docstring
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_pydoc_multiline_minimal() {
    let input = r#"
def test():
    """
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_pydoc_class_docstring() {
    let input = r#"
class MyClass:
    """Class docstring.

    Attributes:
        attr1: First attribute
        attr2: Second attribute
    """

    def method(self):
        """Method docstring."""
        pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_module_docstring() {
    let input = r#""""Module docstring at the start.

This module does something useful.
"""

def func():
    """Function docstring."""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_escaped_quotes() {
    let input = r#"
def test():
    """Contains \"escaped\" quotes and \'single\' too."""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_raw_string() {
    let input = r#"
def test():
    r"""Raw docstring with \n and \t."""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    // Raw strings might not be recognized, but shouldn't crash
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_pydoc_consecutive() {
    let input = r#"
def first():
    """First docstring."""
    pass

def second():
    """Second docstring."""
    pass

def third():
    """Third docstring."""
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 3);
  }

  #[test]
  fn test_pydoc_with_code_block() {
    let input = r#"
def test():
    """Description.

    Example::

        >>> x = 1
        >>> y = 2
        >>> x + y
        3

    More text after code.
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_type_hints_in_doc() {
    let input = r#"
def test():
    """
    Args:
        x (int | str): Union type
        y (list[dict[str, Any]]): Complex generic
        z (Optional[Callable[[int], str]]): Optional callable

    Returns:
        tuple[int, str, float]: Multiple return types
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_indentation_variations() {
    let input = r#"
def test():
    """
    Args:
       param1: Three space indent
        param2: Four space indent
          param3: Six space indent
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_special_sections() {
    let input = r#"
def test():
    """Summary.

    Todo:
        * Item 1
        * Item 2

    Deprecated:
        Use other_func instead.

    .. deprecated:: 1.0
       Use new_func instead.
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_only_code() {
    let input = "def no_docstring(): pass\n\nx = 1 + 2";
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_string_not_docstring() {
    let input = r#"
x = """This is a string, not a docstring"""

def test():
    x = """Also just a string"""
    return x
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    // Parser may or may not pick these up - just shouldn't crash
    let _ = doc.nodes.len();
  }

  #[test]
  fn test_pydoc_async_function() {
    let input = r#"
async def async_func():
    """Async function docstring.

    Returns:
        Awaitable result.
    """
    pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_pydoc_deeply_nested() {
    let input = r#"
class Outer:
    """Outer class."""

    class Inner:
        """Inner class."""

        def method(self):
            """Inner method."""

            def nested():
                """Nested function."""
                pass
"#;
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.len() >= 2);
  }

  // ============================================
  // CROSS-PARSER EDGE CASES
  // ============================================

  #[test]
  fn test_very_long_docstring() {
    let long_text = "x".repeat(10000);
    let input = format!("/**\n * {}\n */", long_text);
    let mut parser = JsDocParser::new(&input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_many_comments() {
    let comments: String = (0..100)
      .map(|i| format!("/** Comment {} */\n", i))
      .collect();
    let mut parser = JsDocParser::new(&comments);
    let doc = parser.parse();
    assert_eq!(doc.nodes.len(), 100);
  }

  #[test]
  fn test_only_whitespace_js() {
    let input = "     \n\n\t\t\n     ";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_only_whitespace_java() {
    let input = "     \n\n\t\t\n     ";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_only_whitespace_py() {
    let input = "     \n\n\t\t\n     ";
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(doc.nodes.is_empty());
  }

  #[test]
  fn test_null_bytes_js() {
    let input = "/** Has \0 null */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_null_bytes_java() {
    let input = "/** Has \0 null */";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_mixed_line_endings_js() {
    let input = "/**\r\n * Line 1\r\n * Line 2\n * Line 3\r */";
    let mut parser = JsDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_mixed_line_endings_java() {
    let input = "/**\r\n * Line 1\r\n * Line 2\n * Line 3\r */";
    let mut parser = JavaDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }

  #[test]
  fn test_mixed_line_endings_py() {
    let input = "def f():\r\n    \"\"\"Line 1\r\n    Line 2\n    Line 3\r    \"\"\"\r\n    pass";
    let mut parser = PyDocParser::new(input);
    let doc = parser.parse();
    assert!(!doc.nodes.is_empty());
  }
}
