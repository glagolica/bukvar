"""
Sample Python module with docstrings.

This module demonstrates various Python documentation styles
including Google-style and NumPy-style docstrings.
"""


def calculate_area(width: float, height: float) -> float:
    """Calculate the area of a rectangle.

    Args:
        width: The width of the rectangle.
        height: The height of the rectangle.

    Returns:
        The calculated area.

    Raises:
        ValueError: If width or height is negative.

    Example:
        >>> calculate_area(5, 10)
        50.0
    """
    if width < 0 or height < 0:
        raise ValueError("Dimensions cannot be negative")
    return width * height


class DataProcessor:
    """A class for processing data.

    This class provides methods for loading, transforming,
    and saving data in various formats.

    Attributes:
        data: The loaded data.
        format: The data format (csv, json, etc).

    Example:
        processor = DataProcessor()
        processor.load("data.csv")
        processor.transform()
        processor.save("output.json")
    """

    def __init__(self):
        """Initialize the DataProcessor."""
        self.data = None
        self.format = None

    def load(self, filepath: str) -> None:
        """Load data from a file.

        Parameters
        ----------
        filepath : str
            Path to the input file.

        Notes
        -----
        Supported formats: CSV, JSON, XML.
        """
        pass

    def transform(self, operations: list = None) -> None:
        """Apply transformations to the data.

        Parameters
        ----------
        operations : list, optional
            List of operations to apply.
            Default is None (no operations).

        See Also
        --------
        load : Load data before transforming.
        save : Save data after transforming.
        """
        pass
