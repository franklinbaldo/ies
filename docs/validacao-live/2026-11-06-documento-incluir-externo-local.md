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
