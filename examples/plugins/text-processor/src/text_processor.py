#!/usr/bin/env python3
"""
Text Processor Plugin for iTools

This plugin provides comprehensive text processing capabilities including
analysis, format conversion, and content extraction.
"""

import re
import json
import html
import urllib.parse
from typing import Dict, List, Any, Optional
from datetime import datetime


class TextProcessor:
    """Main text processor class"""
    
    def __init__(self):
        self.name = "Text Processor"
        self.version = "1.0.0"
        
        # Regex patterns for content extraction
        self.patterns = {
            'emails': r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b',
            'urls': r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+',
            'phone_numbers': r'(\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})',
            'dates': r'\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b|\b\d{4}[/-]\d{1,2}[/-]\d{1,2}\b',
            'numbers': r'\b\d+\.?\d*\b'
        }

    async def initialize(self) -> Dict[str, Any]:
        """Initialize the plugin"""
        print(f"{self.name} v{self.version} initialized")
        return {
            "status": "success",
            "message": "Text processor plugin initialized successfully"
        }

    async def handle_tool_call(self, tool_name: str, parameters: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tool calls from the MCP protocol"""
        try:
            if tool_name == 'analyze_text':
                return await self.analyze_text(
                    parameters['text'], 
                    parameters.get('metrics', ['word_count', 'char_count'])
                )
            elif tool_name == 'convert_format':
                return await self.convert_format(
                    parameters['text'],
                    parameters['from_format'],
                    parameters['to_format']
                )
            elif tool_name == 'extract_content':
                return await self.extract_content(
                    parameters['text'],
                    parameters.get('content_types', ['emails', 'urls'])
                )
            else:
                raise ValueError(f"Unknown tool: {tool_name}")
        except Exception as error:
            return {
                "status": "error",
                "error": str(error)
            }

    async def analyze_text(self, text: str, metrics: List[str]) -> Dict[str, Any]:
        """Analyze text for various metrics"""
        results = {}
        
        if 'word_count' in metrics:
            words = text.split()
            results['word_count'] = len(words)
        
        if 'char_count' in metrics:
            results['char_count'] = len(text)
            results['char_count_no_spaces'] = len(text.replace(' ', ''))
        
        if 'readability' in metrics:
            results['readability'] = self._calculate_readability(text)
        
        if 'sentiment' in metrics:
            results['sentiment'] = self._analyze_sentiment(text)
        
        if 'keywords' in metrics:
            results['keywords'] = self._extract_keywords(text)
        
        # Additional basic metrics
        lines = text.split('\n')
        sentences = re.split(r'[.!?]+', text)
        paragraphs = [p.strip() for p in text.split('\n\n') if p.strip()]
        
        results.update({
            'line_count': len(lines),
            'sentence_count': len([s for s in sentences if s.strip()]),
            'paragraph_count': len(paragraphs),
            'average_words_per_sentence': results.get('word_count', 0) / max(len([s for s in sentences if s.strip()]), 1)
        })
        
        return {
            "status": "success",
            "result": {
                "text_preview": text[:100] + "..." if len(text) > 100 else text,
                "metrics": results,
                "analysis_timestamp": datetime.now().isoformat()
            }
        }

    async def convert_format(self, text: str, from_format: str, to_format: str) -> Dict[str, Any]:
        """Convert text between different formats"""
        try:
            if from_format == to_format:
                converted_text = text
            elif from_format == 'plain' and to_format == 'html':
                converted_text = self._plain_to_html(text)
            elif from_format == 'html' and to_format == 'plain':
                converted_text = self._html_to_plain(text)
            elif from_format == 'plain' and to_format == 'markdown':
                converted_text = self._plain_to_markdown(text)
            elif from_format == 'markdown' and to_format == 'html':
                converted_text = self._markdown_to_html(text)
            elif from_format == 'plain' and to_format == 'json':
                converted_text = json.dumps({"content": text}, indent=2)
            elif from_format == 'json' and to_format == 'plain':
                data = json.loads(text)
                converted_text = str(data)
            else:
                raise ValueError(f"Conversion from {from_format} to {to_format} not supported")
            
            return {
                "status": "success",
                "result": {
                    "original_format": from_format,
                    "target_format": to_format,
                    "converted_text": converted_text,
                    "original_length": len(text),
                    "converted_length": len(converted_text)
                }
            }
        except Exception as e:
            raise ValueError(f"Format conversion error: {str(e)}")

    async def extract_content(self, text: str, content_types: List[str]) -> Dict[str, Any]:
        """Extract specific content from text"""
        results = {}
        
        for content_type in content_types:
            if content_type in self.patterns:
                pattern = self.patterns[content_type]
                matches = re.findall(pattern, text)
                results[content_type] = list(set(matches))  # Remove duplicates
            else:
                results[content_type] = []
        
        return {
            "status": "success",
            "result": {
                "extracted_content": results,
                "total_matches": sum(len(matches) for matches in results.values()),
                "extraction_timestamp": datetime.now().isoformat()
            }
        }

    def _calculate_readability(self, text: str) -> Dict[str, float]:
        """Calculate basic readability metrics"""
        words = text.split()
        sentences = re.split(r'[.!?]+', text)
        sentences = [s for s in sentences if s.strip()]
        
        if not words or not sentences:
            return {"flesch_reading_ease": 0, "grade_level": 0}
        
        avg_sentence_length = len(words) / len(sentences)
        
        # Simple syllable counting (approximation)
        syllables = sum(self._count_syllables(word) for word in words)
        avg_syllables_per_word = syllables / len(words)
        
        # Flesch Reading Ease (simplified)
        flesch_score = 206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables_per_word)
        
        # Approximate grade level
        grade_level = 0.39 * avg_sentence_length + 11.8 * avg_syllables_per_word - 15.59
        
        return {
            "flesch_reading_ease": max(0, min(100, flesch_score)),
            "grade_level": max(0, grade_level),
            "avg_sentence_length": avg_sentence_length,
            "avg_syllables_per_word": avg_syllables_per_word
        }

    def _count_syllables(self, word: str) -> int:
        """Simple syllable counting"""
        word = word.lower()
        vowels = 'aeiouy'
        syllable_count = 0
        previous_was_vowel = False
        
        for char in word:
            is_vowel = char in vowels
            if is_vowel and not previous_was_vowel:
                syllable_count += 1
            previous_was_vowel = is_vowel
        
        # Handle silent 'e'
        if word.endswith('e'):
            syllable_count -= 1
        
        return max(1, syllable_count)

    def _analyze_sentiment(self, text: str) -> Dict[str, Any]:
        """Basic sentiment analysis"""
        positive_words = ['good', 'great', 'excellent', 'amazing', 'wonderful', 'fantastic', 'love', 'like', 'happy', 'joy']
        negative_words = ['bad', 'terrible', 'awful', 'horrible', 'hate', 'dislike', 'sad', 'angry', 'disappointed', 'frustrated']
        
        words = text.lower().split()
        positive_count = sum(1 for word in words if word in positive_words)
        negative_count = sum(1 for word in words if word in negative_words)
        
        total_sentiment_words = positive_count + negative_count
        
        if total_sentiment_words == 0:
            sentiment = "neutral"
            score = 0.0
        elif positive_count > negative_count:
            sentiment = "positive"
            score = positive_count / total_sentiment_words
        elif negative_count > positive_count:
            sentiment = "negative"
            score = -negative_count / total_sentiment_words
        else:
            sentiment = "neutral"
            score = 0.0
        
        return {
            "sentiment": sentiment,
            "score": score,
            "positive_words": positive_count,
            "negative_words": negative_count
        }

    def _extract_keywords(self, text: str, max_keywords: int = 10) -> List[str]:
        """Extract keywords from text"""
        # Simple keyword extraction based on word frequency
        words = re.findall(r'\b\w+\b', text.lower())
        
        # Filter out common stop words
        stop_words = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by', 'is', 'are', 'was', 'were', 'be', 'been', 'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would', 'could', 'should', 'may', 'might', 'must', 'can', 'this', 'that', 'these', 'those', 'i', 'you', 'he', 'she', 'it', 'we', 'they', 'me', 'him', 'her', 'us', 'them'}
        
        filtered_words = [word for word in words if word not in stop_words and len(word) > 2]
        
        # Count word frequency
        word_freq = {}
        for word in filtered_words:
            word_freq[word] = word_freq.get(word, 0) + 1
        
        # Sort by frequency and return top keywords
        sorted_words = sorted(word_freq.items(), key=lambda x: x[1], reverse=True)
        return [word for word, freq in sorted_words[:max_keywords]]

    def _plain_to_html(self, text: str) -> str:
        """Convert plain text to HTML"""
        # Escape HTML characters
        escaped = html.escape(text)
        # Convert line breaks to <br> tags
        return escaped.replace('\n', '<br>\n')

    def _html_to_plain(self, text: str) -> str:
        """Convert HTML to plain text"""
        # Remove HTML tags
        clean = re.sub(r'<[^>]+>', '', text)
        # Unescape HTML entities
        return html.unescape(clean)

    def _plain_to_markdown(self, text: str) -> str:
        """Convert plain text to Markdown"""
        # Simple conversion - wrap paragraphs
        paragraphs = text.split('\n\n')
        return '\n\n'.join(paragraphs)

    def _markdown_to_html(self, text: str) -> str:
        """Basic Markdown to HTML conversion"""
        # This is a very basic implementation
        html_text = text
        
        # Headers
        html_text = re.sub(r'^# (.+)$', r'<h1>\1</h1>', html_text, flags=re.MULTILINE)
        html_text = re.sub(r'^## (.+)$', r'<h2>\1</h2>', html_text, flags=re.MULTILINE)
        html_text = re.sub(r'^### (.+)$', r'<h3>\1</h3>', html_text, flags=re.MULTILINE)
        
        # Bold and italic
        html_text = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', html_text)
        html_text = re.sub(r'\*(.+?)\*', r'<em>\1</em>', html_text)
        
        # Line breaks
        html_text = html_text.replace('\n', '<br>\n')
        
        return html_text

    async def handle_prompt(self, prompt_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Handle prompt requests"""
        if prompt_name == 'summarize_text':
            text = arguments['text']
            max_length = arguments.get('max_length', 100)
            
            # Simple summarization by taking first sentences
            sentences = re.split(r'[.!?]+', text)
            summary_sentences = []
            word_count = 0
            
            for sentence in sentences:
                sentence = sentence.strip()
                if sentence:
                    sentence_words = len(sentence.split())
                    if word_count + sentence_words <= max_length:
                        summary_sentences.append(sentence)
                        word_count += sentence_words
                    else:
                        break
            
            summary = '. '.join(summary_sentences) + '.'
            
            return {
                "status": "success",
                "result": {
                    "summary": summary,
                    "original_length": len(text.split()),
                    "summary_length": len(summary.split()),
                    "compression_ratio": len(summary.split()) / len(text.split())
                }
            }
        else:
            raise ValueError(f"Unknown prompt: {prompt_name}")

    def get_capabilities(self) -> Dict[str, List[str]]:
        """Get plugin capabilities"""
        return {
            "tools": ['analyze_text', 'convert_format', 'extract_content'],
            "resources": ['templates/report.html'],
            "prompts": ['summarize_text']
        }

    async def cleanup(self) -> Dict[str, Any]:
        """Cleanup when plugin is unloaded"""
        print(f"{self.name} cleanup completed")
        return {
            "status": "success",
            "message": "Text processor plugin cleanup completed"
        }


# Plugin entry point for iTools
async def create_plugin():
    """Create and initialize the plugin"""
    processor = TextProcessor()
    await processor.initialize()
    return processor


if __name__ == "__main__":
    # Test the plugin
    import asyncio
    
    async def test():
        processor = await create_plugin()
        
        # Test text analysis
        result = await processor.analyze_text(
            "This is a sample text for testing. It has multiple sentences! How wonderful?",
            ["word_count", "char_count", "readability", "sentiment"]
        )
        print("Analysis result:", json.dumps(result, indent=2))
        
        # Test format conversion
        result = await processor.convert_format(
            "Hello\nWorld", "plain", "html"
        )
        print("Conversion result:", json.dumps(result, indent=2))
        
        # Test content extraction
        result = await processor.extract_content(
            "Contact us at support@example.com or visit https://example.com for more info. Call (555) 123-4567.",
            ["emails", "urls", "phone_numbers"]
        )
        print("Extraction result:", json.dumps(result, indent=2))
    
    asyncio.run(test())
