/**
 * HTTP Client for REST API communication.
 *
 * A lightweight HTTP client with retry logic, request/response
 * interceptors, and automatic JSON parsing.
 *
 * @module http-client
 * @version 1.2.0
 * @author API Team
 * @license MIT
 */

/**
 * @typedef {Object} RequestConfig
 * @property {string} url - The request URL
 * @property {string} [method='GET'] - HTTP method
 * @property {Object.<string, string>} [headers] - Request headers
 * @property {*} [body] - Request body (will be JSON stringified)
 * @property {number} [timeout=30000] - Request timeout in ms
 * @property {boolean} [withCredentials=false] - Include credentials
 */

/**
 * @typedef {Object} Response
 * @property {number} status - HTTP status code
 * @property {string} statusText - HTTP status text
 * @property {Object.<string, string>} headers - Response headers
 * @property {*} data - Parsed response body
 * @property {RequestConfig} config - Original request config
 */

/**
 * @typedef {Object} HttpError
 * @property {string} message - Error message
 * @property {number} [status] - HTTP status if available
 * @property {Response} [response] - Full response if available
 * @property {RequestConfig} config - Original request config
 */

/**
 * @callback RequestInterceptor
 * @param {RequestConfig} config - The request configuration
 * @returns {RequestConfig|Promise<RequestConfig>} Modified config
 */

/**
 * @callback ResponseInterceptor
 * @param {Response} response - The response object
 * @returns {Response|Promise<Response>} Modified response
 */

/**
 * HTTP Client class for making API requests.
 *
 * @class
 * @example
 * const client = new HttpClient({ baseURL: 'https://api.example.com' });
 *
 * // Add auth token to all requests
 * client.interceptors.request.use(config => {
 *   config.headers['Authorization'] = `Bearer ${getToken()}`;
 *   return config;
 * });
 *
 * // GET request
 * const users = await client.get('/users');
 *
 * // POST request
 * const newUser = await client.post('/users', { name: 'John' });
 */
class HttpClient {
  /**
   * Creates a new HTTP client instance.
   *
   * @param {Object} [options] - Client configuration
   * @param {string} [options.baseURL=''] - Base URL for all requests
   * @param {number} [options.timeout=30000] - Default timeout
   * @param {Object.<string, string>} [options.headers] - Default headers
   */
  constructor(options = {}) {
    this.baseURL = options.baseURL || "";
    this.timeout = options.timeout || 30000;
    this.defaultHeaders = options.headers || {};

    this.interceptors = {
      request: new InterceptorManager(),
      response: new InterceptorManager(),
    };
  }

  /**
   * Performs a GET request.
   *
   * @param {string} url - Request URL (appended to baseURL)
   * @param {Object} [config] - Additional request config
   * @returns {Promise<Response>} The response
   * @throws {HttpError} If request fails
   *
   * @example
   * // Simple GET
   * const data = await client.get('/users');
   *
   * // With query params
   * const filtered = await client.get('/users?role=admin');
   */
  async get(url, config = {}) {
    return this.request({ ...config, url, method: "GET" });
  }

  /**
   * Performs a POST request.
   *
   * @param {string} url - Request URL
   * @param {*} [data] - Request body
   * @param {Object} [config] - Additional config
   * @returns {Promise<Response>} The response
   * @throws {HttpError} If request fails
   */
  async post(url, data, config = {}) {
    return this.request({ ...config, url, method: "POST", body: data });
  }

  /**
   * Performs a PUT request.
   *
   * @param {string} url - Request URL
   * @param {*} [data] - Request body
   * @param {Object} [config] - Additional config
   * @returns {Promise<Response>} The response
   */
  async put(url, data, config = {}) {
    return this.request({ ...config, url, method: "PUT", body: data });
  }

  /**
   * Performs a DELETE request.
   *
   * @param {string} url - Request URL
   * @param {Object} [config] - Additional config
   * @returns {Promise<Response>} The response
   */
  async delete(url, config = {}) {
    return this.request({ ...config, url, method: "DELETE" });
  }

  /**
   * Core request method.
   *
   * @private
   * @param {RequestConfig} config - Full request configuration
   * @returns {Promise<Response>} The response
   * @throws {HttpError} If request fails or times out
   */
  async request(config) {
    // Apply request interceptors
    let finalConfig = { ...config };
    for (const interceptor of this.interceptors.request.handlers) {
      finalConfig = await interceptor(finalConfig);
    }

    const url = this.baseURL + finalConfig.url;
    const headers = { ...this.defaultHeaders, ...finalConfig.headers };

    const controller = new AbortController();
    const timeoutId = setTimeout(
      () => controller.abort(),
      finalConfig.timeout || this.timeout
    );

    try {
      const response = await fetch(url, {
        method: finalConfig.method || "GET",
        headers,
        body: finalConfig.body ? JSON.stringify(finalConfig.body) : undefined,
        signal: controller.signal,
        credentials: finalConfig.withCredentials ? "include" : "same-origin",
      });

      clearTimeout(timeoutId);

      let data;
      const contentType = response.headers.get("content-type");
      if (contentType?.includes("application/json")) {
        data = await response.json();
      } else {
        data = await response.text();
      }

      let result = {
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        data,
        config: finalConfig,
      };

      // Apply response interceptors
      for (const interceptor of this.interceptors.response.handlers) {
        result = await interceptor(result);
      }

      if (!response.ok) {
        throw this.createError("Request failed", result);
      }

      return result;
    } catch (error) {
      clearTimeout(timeoutId);
      if (error.name === "AbortError") {
        throw this.createError("Request timeout", null, finalConfig);
      }
      throw error;
    }
  }

  /**
   * Creates an HttpError object.
   *
   * @private
   * @param {string} message - Error message
   * @param {Response} [response] - Response if available
   * @param {RequestConfig} [config] - Request config
   * @returns {HttpError} The error object
   */
  createError(message, response, config) {
    return {
      message,
      status: response?.status,
      response,
      config: config || response?.config,
    };
  }
}

/**
 * Manages request/response interceptors.
 *
 * @class
 * @private
 */
class InterceptorManager {
  constructor() {
    /** @type {Function[]} */
    this.handlers = [];
  }

  /**
   * Adds an interceptor.
   *
   * @param {Function} handler - Interceptor function
   * @returns {number} Interceptor ID for removal
   */
  use(handler) {
    this.handlers.push(handler);
    return this.handlers.length - 1;
  }

  /**
   * Removes an interceptor.
   *
   * @param {number} id - Interceptor ID from use()
   */
  eject(id) {
    if (this.handlers[id]) {
      this.handlers[id] = null;
    }
  }
}

/**
 * Creates a pre-configured HTTP client instance.
 *
 * @param {Object} config - Client configuration
 * @returns {HttpClient} Configured client instance
 *
 * @example
 * const api = createClient({
 *   baseURL: 'https://api.example.com/v1',
 *   timeout: 10000,
 *   headers: { 'X-API-Key': 'secret' }
 * });
 */
function createClient(config) {
  return new HttpClient(config);
}

module.exports = { HttpClient, createClient };
