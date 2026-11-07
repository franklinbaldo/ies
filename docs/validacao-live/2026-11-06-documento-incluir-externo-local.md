---
type: SEIObservation
title: Pré-inspeção local de PDF externo — issue 332
description: Evidências test-first da inspeção offline sem sessão, upload, escrita ou ensaio de produção
---

# #332 — inspeção local de PDF antes do envio

## Origem do comportamento

O arquivo `franklinbaldo/todos/src/todos/documento_externo.py` documenta observação prévia em **UnB TREINAMENTO, SEI 4**, em 11/10/2026: cadastro `documento_receber`, upload temporário `documento_upload_anexo`, recibo, `hdnAnexos` e confirmação remota por releitura. Isto é **oráculo anterior do Todos**, não ensaio desta fatia nem confirmação da instalação atual.

## Testes sintéticos locais

- **RED:** [GitHub Actions 42330589633](https://github.com/franklinbaldo/ies/actions/runs/42330589633), commit `2474a7a`, **6 FAIL novos, 1.782 PASS, 22 ignored preexistentes**; os testes foram publicados com `inspecionar_pdf` ainda propositalmente retornando `Ausente`. O mesmo run registrou Rustfmt/Clippy/subcódigos falhando pelo esqueleto, não tratá-lo como GREEN.
- **GREEN:** implementação publicada posteriormente com inspeção offline tipada, SHA-256 em blocos e recusa de arquivo não regular, symlink, extensão, assinatura, vazio e limite observado. **CI final deve ser conferido no SHA exato da PR #352**; não atribuir PASS antes do gate.
- **Orçamento:** todos os testes novos operam `std::fs` com arquivos sintéticos e não acessam SEI. A CLI não abre sessão nem rede.

## RED adicional — validação local do nome

Em 07/11/2026 foram acrescentados três cenários: nomes contendo `&#` (incompatíveis com o upload observado no Todos), sequência de escape de terminal no nome em Unix e rejeição pela CLI antes da sessão. O RED [run 42334803924](https://github.com/franklinbaldo/ies/actions/runs/42334803924), head `d558491`, registrou no teste unitário **1.805 PASS, 2 FAIL, 23 ignored**, com `Rustfmt` e `Clippy` aprovados. O teste CLI fica para o estágio de integração após a correção. GREEN deve ser comprovado em run posterior do head final; nenhum teste RED é considerado PASS antecipadamente.

## Retificação do contrato de erros — ADR 0005 (07/11/2026)

A fatia anterior acrescentara nomes textuais de subcódigos, embora novas
fatias devam publicar o erro tipado por Serde. Correção na mesma PR #352:
`FalhaPdf::subcodigo()` removida; CLI publica `causa` diretamente,
sem tabela paralela. Os testes de identidade voltaram a comparar variantes.

- **RED:** [CI 42335719860](https://github.com/franklinbaldo/ies/actions/runs/42335719860),
  head `c5c78fc`: 5 falhas novas no teste de CLI, 1 teste de CLI
  aprovado; 1.807 testes unitários aprovados, 23 ignorados já existentes.
  O hook Rustfmt também acusou a formatação da etapa RED.
- **GREEN:** candidato no commit `07a89d8` ([CI 42335957617](https://github.com/franklinbaldo/ies/actions/runs/42335957617)).
  Consultar o resultado do SHA final após ajustes documentais; esta linha
  não presume sucesso antes de todos os gates.
- **Live remoto:** NOT_RUN; nenhuma sessão, GET, POST, upload, escrita ou
  resíduo SEI. A confirmação continua recusada antes da rede.

## RED → GREEN adicional — integridade da apresentação (07/11/2026)

- **RED publicado antes da correção:** commit `c76ac72`, [run CI 42339576715](https://github.com/franklinbaldo/ies/actions/runs/42339576715). Suite Rust: **1.810 PASS, 1 FAIL novo, 23 ignored**. O teste unitário demonstrou que `char::is_control` aceitava um nome com U+202E e devolvia `PdfLocal`, em vez de `FalhaPdf::NomeInvalido`. O teste CLI para tipo com U+202E foi publicado no mesmo RED, mas o hook não o executou após a falha unitária.
- **GREEN em código:** usar o mesmo predicado de apresentação para nomes de PDF e texto do tipo solicitado; rejeitar controles e marcadores bidirecionais Unicode (U+061C, U+200E/F, U+202A–E e U+2066–9). São **caracteres que reordenam texto na saída humana**, não um limite de charset alegado pelo SEI. Os erros existentes `Pdf(NomeInvalido)` e `Tipo(Controle)` permanecem tipados; sem novo subcódigo. `--confirmar` continua recusado antes da sessão/rede.
- **CI intermediário:** [42339719537](https://github.com/franklinbaldo/ies/actions/runs/42339719537) verificou Rust/Clippy/OKF com PASS, mas falhou somente em Rustfmt de três trechos; correções de formatação foram commitadas antes do head final. **Consultar o CI do head atual** para atestar GREEN definitivo.
- **Acervo adicional lido, sem rede SEI:** [cassette PEN3 de prévia](../../cassettes/pen3/documento-juntar-previa-pen3--ec754357a3c7.md) (SEI 5.0.4: 9 interações na captura, 5 GET + 4 POST), [testes UnB 4](https://github.com/franklinbaldo/todos/blob/main/tests/test_documento_juntar_010.py) e [SEI 5](https://github.com/franklinbaldo/todos/blob/main/tests/test_documento_juntar_sei5_010.py). A evidência de versões diferentes, recibo e duas mutações está detalhada no [cartão da operação](../../contracts/evidencias/documento-incluir-externo.md). Contagens dessas capturas **não** são requisições desta execução do IES.

## Ensaio ao vivo desta fatia

| Etapa | Resultado | Requisições reais | Resíduo |
| --- | --- | ---: | --- |
| Login | NOT_RUN | 0 | nenhum |
| Prévia remota do cadastro | NOT_RUN | 0 | nenhum |
| Upload temporário | NOT_RUN | 0 | nenhum |
| Cadastro definitivo | NOT_RUN | 0 | nenhum |
| Releitura independente | NOT_RUN | 0 | nenhum |

**Total real nesta fatia: 0 requisições ao SEI, 0 GET, 0 POST, 0 escrita e nenhum resíduo.** Não houve acesso à produção nem a treinamento. O comando `--confirmar` permanece deliberadamente recusado antes de abrir a sessão ou a rede.

## Próxima etapa da issue

Observar cadastro e limites da tela nas instalações de treinamento; modelar as duas etapas mutadoras e a verificação efetiva do anexo sem confiar apenas na árvore; fazer RED de transporte, GREEN, CI, prévia viva imediatamente antes de **um comando confirmado**, releitura e registro do resíduo, sem retry. Não declarar a #332 concluída.
