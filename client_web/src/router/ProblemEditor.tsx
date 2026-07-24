import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Button,
  Input,
  Textarea,
  Field,
  Select,
  Card,
  Text,
  Spinner,
} from '@fluentui/react-components';
import { ArrowUploadRegular, DeleteRegular, AddRegular } from '@fluentui/react-icons';
import ReactMarkdown from 'react-markdown';
import rehypeKatex from 'rehype-katex';
import remarkMath from 'remark-math';
import 'katex/dist/katex.min.css';

const CONTROL_PANEL_URL = 'http://localhost:9990';

interface TestCase {
  input_file: string;
  output_file: string;
  time_limit: number;
  memory_limit: number;
  score: number;
}

interface ProblemEditorProps {
  isOpen: boolean;
  onClose: () => void;
  problemNumber?: number;
  onSuccess: () => void;
}

export default function ProblemEditor({ isOpen, onClose, problemNumber, onSuccess }: ProblemEditorProps) {
  const { t } = useTranslation(['problemEditor', 'common']);
  const [activeTab, setActiveTab] = useState<'basic' | 'statement' | 'testdata'>('basic');
  const [loading, setLoading] = useState(false);

  // Basic info
  const [newProblemNumber, setNewProblemNumber] = useState('');
  const [problemName, setProblemName] = useState('');
  const [difficulty, setDifficulty] = useState('1');
  const [timeLimit, setTimeLimit] = useState('1000');
  const [memoryLimit, setMemoryLimit] = useState('256');

  // Problem statement
  const [statement, setStatement] = useState('');
  const [showPreview, setShowPreview] = useState(false);

  // Test data
  const [testCases, setTestCases] = useState<TestCase[]>([]);
  const [uploadingData, setUploadingData] = useState(false);
  // Cache the title's problem number so closing animation keeps showing the
  // correct title instead of briefly flipping to "Create New Problem".
  const [displayNumber, setDisplayNumber] = useState<number | undefined>(undefined);

  useEffect(() => {
    if (isOpen) {
      setDisplayNumber(problemNumber);
      if (problemNumber) {
        loadProblem();
      } else {
        resetForm();
      }
    }
  }, [isOpen, problemNumber]);

  const loadProblem = async () => {
    if (!problemNumber) return;

    setLoading(true);
    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/problems/${problemNumber}`);
      const data = await response.json();

      if (data.ok) {
        const problem = data.problem;
        setProblemName(problem.problem_name);
        setDifficulty(problem.difficulty.toString());

        // problem_statement is stored as a JSON array of strings (one per
        // paragraph); the site renders it by joining with newlines. Show it as
        // editable multi-line text, and split it back to an array on save.
        let stmtText = '';
        try {
          const arr = JSON.parse(problem.problem_statement);
          stmtText = Array.isArray(arr) ? arr.join('\n') : String(problem.problem_statement);
        } catch {
          stmtText = problem.problem_statement || '';
        }
        setStatement(stmtText);

        // time_limit / memory_limit / test_cases live inside testcase_config JSON.
        // Its testcases use fields: number, score, input, answer, time_limit, memory_limit.
        let config: any = {};
        if (problem.testcase_config) {
          try {
            config = JSON.parse(problem.testcase_config);
          } catch {
            config = {};
          }
        }
        const cases = Array.isArray(config.testcases) ? config.testcases : [];
        setTestCases(
          cases.map((tc: any) => ({
            input_file: tc.input || '',
            output_file: tc.answer || '',
            time_limit: Math.round((tc.time_limit ?? 1) * 1000),
            memory_limit: tc.memory_limit ?? 256,
            score: tc.score ?? 0,
          }))
        );
        // Default the top-level limits to the first case, if any.
        setTimeLimit((cases[0] ? Math.round((cases[0].time_limit ?? 1) * 1000) : 1000).toString());
        setMemoryLimit((cases[0]?.memory_limit ?? 256).toString());
      }
    } catch (err) {
      console.error('Failed to load problem:', err);
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setNewProblemNumber('');
    setProblemName('');
    setDifficulty('1');
    setTimeLimit('1000');
    setMemoryLimit('256');
    setStatement('');
    setTestCases([]);
    setActiveTab('basic');
  };

  const handleSave = async () => {
    setLoading(true);
    try {
      // Statement is stored as a JSON array of paragraphs (split on newlines).
      const statementArray = statement.split('\n');
      const problem_statement = JSON.stringify(statementArray);

      // testcase_config matches the judge's schema: a `testcases` array whose
      // entries carry number/score/input/answer/time_limit(sec)/memory_limit(MB).
      const testcase_config = JSON.stringify({
        testcases: testCases.map((tc, i) => ({
          number: i + 1,
          score: tc.score,
          input: tc.input_file,
          answer: tc.output_file,
          time_limit: tc.time_limit / 1000,
          memory_limit: tc.memory_limit,
        })),
      });

      let url: string;
      let method: string;
      let payload: any;

      if (problemNumber) {
        url = `${CONTROL_PANEL_URL}/api/problems/${problemNumber}`;
        method = 'PUT';
        payload = {
          problem_name: problemName,
          difficulty: parseInt(difficulty),
          problem_statement,
          testcase_config,
        };
      } else {
        url = `${CONTROL_PANEL_URL}/api/problems/create`;
        method = 'POST';
        payload = {
          problem_number: parseInt(newProblemNumber),
          problem_name: problemName,
          difficulty: parseInt(difficulty),
          problem_statement,
          testcase_config,
        };
      }

      const response = await fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      const data = await response.json();
      if (data.ok) {
        onSuccess();
        onClose();
      } else {
        alert(t('message.saveFailedWithReason', { message: data.message }));
      }
    } catch (err) {
      alert(t('message.saveFailedGeneric'));
    } finally {
      setLoading(false);
    }
  };

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>, type: 'input' | 'output', index: number) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setUploadingData(true);
    try {
      const formData = new FormData();
      formData.append('file', file);
      formData.append('problem_number', problemNumber?.toString() || 'new');
      formData.append('type', type);

      const response = await fetch(`${CONTROL_PANEL_URL}/api/upload-testdata`, {
        method: 'POST',
        body: formData,
      });

      const data = await response.json();
      if (data.ok) {
        const newTestCases = [...testCases];
        if (type === 'input') {
          newTestCases[index].input_file = data.filename;
        } else {
          newTestCases[index].output_file = data.filename;
        }
        setTestCases(newTestCases);
      } else {
        alert(t('message.uploadFailedWithReason', { message: data.message }));
      }
    } catch (err) {
      alert(t('message.uploadFailedGeneric'));
    } finally {
      setUploadingData(false);
    }
  };

  const addTestCase = () => {
    setTestCases([
      ...testCases,
      {
        input_file: '',
        output_file: '',
        time_limit: parseInt(timeLimit),
        memory_limit: parseInt(memoryLimit),
        score: 10,
      },
    ]);
  };

  const removeTestCase = (index: number) => {
    setTestCases(testCases.filter((_, i) => i !== index));
  };

  const updateTestCase = (index: number, field: keyof TestCase, value: any) => {
    const newTestCases = [...testCases];
    newTestCases[index] = { ...newTestCases[index], [field]: value };
    setTestCases(newTestCases);
  };

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
      return () => { document.body.style.overflow = ''; };
    }
  }, [isOpen]);

  return (
    <Dialog open={isOpen} onOpenChange={(_, data) => !data.open && onClose()} modalType="non-modal">
      {isOpen && (
        <div
          onClick={onClose}
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.4)',
            zIndex: 1000000,
          }}
        />
      )}
      <DialogSurface style={{ maxWidth: '900px', maxHeight: '90vh', display: 'flex', flexDirection: 'column' }}>
        <DialogTitle>{displayNumber ? t('title.edit', { number: displayNumber }) : t('title.create')}</DialogTitle>
        <DialogBody style={{ display: 'flex', flexDirection: 'column', minHeight: 0, flex: 1 }}>
          <DialogContent style={{ overflowY: 'auto', flex: 1, minHeight: 0, maxHeight: '65vh' }}>
            {loading ? (
              <Spinner label={t('action.loading', { ns: 'common' })} />
            ) : (
              <>
                <div style={{ display: 'flex', gap: '4px', borderBottom: '2px solid #e0e0e0' }}>
                  {(
                    [
                      { value: 'basic', label: t('tabs.basic') },
                      { value: 'statement', label: t('tabs.statement') },
                      { value: 'testdata', label: t('tabs.testdata') },
                    ] as const
                  ).map((tab) => (
                    <button
                      key={tab.value}
                      type="button"
                      onClick={() => setActiveTab(tab.value)}
                      style={{
                        appearance: 'none',
                        background: 'none',
                        border: 'none',
                        cursor: 'pointer',
                        padding: '8px 16px',
                        fontSize: '14px',
                        fontFamily: 'inherit',
                        color: activeTab === tab.value ? '#0f6cbd' : '#242424',
                        borderBottom: activeTab === tab.value ? '2px solid #0f6cbd' : '2px solid transparent',
                        marginBottom: '-2px',
                      }}
                    >
                      {tab.label}
                    </button>
                  ))}
                </div>

                <div style={{ marginTop: '24px' }}>
                  {activeTab === 'basic' && (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                      {!displayNumber && (
                        <Field label={t('basic.problemNumber.label')} required>
                          <Input
                            type="number"
                            value={newProblemNumber}
                            onChange={(_ev, data) => setNewProblemNumber(data.value)}
                            placeholder={t('basic.problemNumber.placeholder')}
                          />
                        </Field>
                      )}
                      <Field label={t('basic.problemName.label')} required>
                        <Input
                          value={problemName}
                          onChange={(_ev, data) => setProblemName(data.value)}
                          placeholder={t('basic.problemName.placeholder')}
                        />
                      </Field>

                      <Field label={t('basic.difficulty.label')} required>
                        <Select value={difficulty} onChange={(e) => setDifficulty(e.target.value)}>
                          <option value="0">{t('basic.difficulty.unknown')}</option>
                          <option value="1">{t('basic.difficulty.beginner')}</option>
                          <option value="2">{t('basic.difficulty.primary')}</option>
                          <option value="3">{t('basic.difficulty.junior')}</option>
                          <option value="4">{t('basic.difficulty.senior')}</option>
                          <option value="5">{t('basic.difficulty.advanced')}</option>
                          <option value="6">{t('basic.difficulty.hard')}</option>
                          <option value="7">{t('basic.difficulty.grand')}</option>
                        </Select>
                      </Field>

                      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
                        <Field label={t('basic.timeLimit.label')}>
                          <Input
                            type="number"
                            value={timeLimit}
                            onChange={(_ev, data) => setTimeLimit(data.value)}
                          />
                        </Field>

                        <Field label={t('basic.memoryLimit.label')}>
                          <Input
                            type="number"
                            value={memoryLimit}
                            onChange={(_ev, data) => setMemoryLimit(data.value)}
                          />
                        </Field>
                      </div>
                    </div>
                  )}

                  {activeTab === 'statement' && (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <Text weight="semibold">{t('statement.label')}</Text>
                        <Button
                          appearance="subtle"
                          onClick={() => setShowPreview(!showPreview)}
                        >
                          {showPreview ? t('action.edit', { ns: 'common' }) : t('statement.preview')}
                        </Button>
                      </div>

                      {showPreview ? (
                        <Card style={{ padding: '16px', maxHeight: '500px', overflow: 'auto' }}>
                          <ReactMarkdown
                            remarkPlugins={[remarkMath]}
                            rehypePlugins={[rehypeKatex]}
                          >
                            {statement}
                          </ReactMarkdown>
                        </Card>
                      ) : (
                        <Textarea
                          value={statement}
                          onChange={(_ev, data) => setStatement(data.value)}
                          placeholder={t('statement.placeholder')}
                          rows={20}
                          style={{ fontFamily: 'monospace' }}
                        />
                      )}

                      <Text size={200} style={{ color: 'gray' }}>
                        {t('statement.tip')}
                      </Text>
                    </div>
                  )}

                  {activeTab === 'testdata' && (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <Text weight="semibold">{t('testdata.title')}</Text>
                        <Button
                          appearance="primary"
                          icon={<AddRegular />}
                          onClick={addTestCase}
                        >
                          {t('testdata.addTestCase')}
                        </Button>
                      </div>

                      {testCases.length === 0 ? (
                        <Text>{t('testdata.empty')}</Text>
                      ) : (
                        testCases.map((tc, index) => (
                          <Card key={index} style={{ padding: '16px' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '12px' }}>
                              <Text weight="semibold">{t('testdata.caseTitle', { number: index + 1 })}</Text>
                              <Button
                                appearance="subtle"
                                icon={<DeleteRegular />}
                                onClick={() => removeTestCase(index)}
                              />
                            </div>

                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px' }}>
                              <Field label={t('testdata.inputFile.label')}>
                                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                                  <Input
                                    value={tc.input_file}
                                    readOnly
                                    placeholder={t('testdata.inputFile.placeholder')}
                                    style={{ flex: 1 }}
                                  />
                                  <Button
                                    appearance="secondary"
                                    icon={<ArrowUploadRegular />}
                                    onClick={() => document.getElementById(`input-${index}`)?.click()}
                                  />
                                  <input
                                    id={`input-${index}`}
                                    type="file"
                                    style={{ display: 'none' }}
                                    onChange={(e) => handleFileUpload(e, 'input', index)}
                                  />
                                </div>
                              </Field>

                              <Field label={t('testdata.outputFile.label')}>
                                <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                                  <Input
                                    value={tc.output_file}
                                    readOnly
                                    placeholder={t('testdata.outputFile.placeholder')}
                                    style={{ flex: 1 }}
                                  />
                                  <Button
                                    appearance="secondary"
                                    icon={<ArrowUploadRegular />}
                                    onClick={() => document.getElementById(`output-${index}`)?.click()}
                                  />
                                  <input
                                    id={`output-${index}`}
                                    type="file"
                                    style={{ display: 'none' }}
                                    onChange={(e) => handleFileUpload(e, 'output', index)}
                                  />
                                </div>
                              </Field>

                              <Field label={t('testdata.timeLimit.label')}>
                                <Input
                                  type="number"
                                  value={tc.time_limit.toString()}
                                  onChange={(_ev, data) => updateTestCase(index, 'time_limit', parseInt(data.value))}
                                />
                              </Field>

                              <Field label={t('testdata.memoryLimit.label')}>
                                <Input
                                  type="number"
                                  value={tc.memory_limit.toString()}
                                  onChange={(_ev, data) => updateTestCase(index, 'memory_limit', parseInt(data.value))}
                                />
                              </Field>

                              <Field label={t('testdata.score.label')}>
                                <Input
                                  type="number"
                                  value={tc.score.toString()}
                                  onChange={(_ev, data) => updateTestCase(index, 'score', parseInt(data.value))}
                                />
                              </Field>
                            </div>
                          </Card>
                        ))
                      )}

                      {uploadingData && <Spinner label={t('testdata.uploading')} />}
                    </div>
                  )}
                </div>
              </>
            )}
          </DialogContent>
        </DialogBody>
        <DialogActions style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid #e0e0e0' }}>
          <Button appearance="secondary" onClick={onClose}>
            {t('action.cancel', { ns: 'common' })}
          </Button>
          <Button appearance="primary" onClick={handleSave} disabled={loading || !problemName}>
            {t('action.save', { ns: 'common' })}
          </Button>
        </DialogActions>
      </DialogSurface>
    </Dialog>
  );
}
