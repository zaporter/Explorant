import React from 'react';
import PropTypes from 'prop-types';

class StringCompletionInput extends React.Component {
  static propTypes = {
    default: PropTypes.string,
    list: PropTypes.arrayOf(PropTypes.string),
    onUpdate: PropTypes.func,
  };

  static defaultProps = {
    default: '',
    list: [],
    onUpdate: () => { },
  };

  constructor(props) {
    super(props);

    this.state = {
      input: props.default,
      suggestions: [],
      showSuggestions: false,
    };

    this.inputRef = React.createRef();

  }

  componentDidMount() {
  }

  onChange = (event) => {
    const input = event.target.value;
    const suggestions = this.props.list.filter((str) => str.startsWith(input));
    if (suggestions.length > 0) {
      this.setState({ input, suggestions, showSuggestions: true });
    }
  };

  onKeyDown = (event) => {
    if ((event.key === 'Tab' | event.key == 'Enter') && this.state.showSuggestions) {
      event.preventDefault();
      this.setState({
        input: this.state.suggestions[0],
        showSuggestions: false,
      });
      this.props.onUpdate(this.state.suggestions[0])
    }
  };

  onBlur = () => {
    // this.props.onUpdate(this.state.input);
    if (this.state.showSuggestions) {
      this.setState({
        input: this.state.suggestions[0],
        showSuggestions: false,
      });
      this.props.onUpdate(this.state.suggestions[0])
    }
  };

  updateSuggestions = () => {
    const suggestions = this.props.list.filter((str) => str.startsWith(this.state.input));
    this.setState({ suggestions, showSuggestions: suggestions.length > 0 });
  };

  render() {
    const { input, suggestions, showSuggestions } = this.state;

    return (
      <div className="string-completion-input">
        <input
          ref={this.inputRef}
          type="text"
          value={input}
          onChange={this.onChange}
          onKeyDown={this.onKeyDown}
          onBlur={this.onBlur}
        />
        {showSuggestions && (
          <ul className="string-completion-suggestions">
            {suggestions.map((suggestion, index) => (
              <li
                key={index}
                onMouseDown={() => {
                  this.setState({ input: suggestion, showSuggestions: false });
                  this.props.onUpdate(suggestion);
                }}
              >
                {suggestion}
              </li>
            ))}
          </ul>
        )}
      </div>
    );

  }
}

export default StringCompletionInput;
