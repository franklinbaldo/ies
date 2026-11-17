---
type: SEIObservation
title: "Cadastro de processo — selects desabilitados não autorizam escrita"
description: "Evidência RED e regressões sintéticas de assunto e hipótese legal desabilitados, sem acesso ao SEI"
operation: processo.criar
date: 2026-11-06
status: not-run
---

# #245 — Controles desabilitados no cadastro de criação

## Observação anterior ao RED

O HTML observado no acervo público do Todos @`323eb01` identifica
`selAssuntos` e `selHipoteseLegal` como seletores do cadastro. O leitor
Rust já recusa `input disabled` e `option disabled`, mas a inspeção de
`conferir_listas` em `src/processo_criar/cadastro.rs` mostrou que ele
aceitava uma `option` de seletor `disabled`: a opção permanece visível
no DOM, mas o próprio seletor não participa da submissão HTML. A hipótese
legal só é exigível quando o nível pedido é `restrito`. Este é um
**caso sintético adversarial**, não uma observação de select desabilitado
nas instalações de treinamento.

## RED test-first

Commit `39383c8`, [CI 42329462594](https://github.com/franklinbaldo/ies/actions/runs/42329462594):
**1 teste novo FAIL, 1.782 PASS, 22 ignorados preexistentes**. A falha
foi aceitar um assunto publicado em seletor desabilitado. O mesmo teste
também exige recusar hipótese legal desabilitada em `restrito` e aceitar
`publico` mesmo quando apenas a hipótese irrelevante está desabilitada.
Nenhum teste preexistente foi enfraquecido ou ignorado.

## GREEN projetado e verificação

Recusar com `ErroCadastro::AssuntoDesabilitado` ou
`ErroCadastro::HipoteseLegalDesabilitada` (causas tipadas específicas)
antes do POST criador, sem inventar valores alternativos nem forçar acesso
público. O teste de fluxo derivado da estrutura do Todos verifica o
orçamento por cenário: **3 GET sintéticos para Controle, lista e cadastro;
0 POST**, inclusive com `--confirmar`. O resultado final dos gates do
head deve ser consultado na PR vinculada, e não inferido deste relatório.

## Ensaio real e resíduo

Prévia em segunda instalação de treinamento **NOT_RUN**;
`--confirmar` único **NOT_RUN**; releitura independente **NOT_RUN**.
**0 requisições reais ao SEI, 0 POST criador, 0 escritas, 0 resíduos novos.**
Não houve login, navegação real ou repetição do processo histórico
inconclusivo da UnB. O RED/GREEN sintético não valida a capacidade live:
a issue permanece aberta até executar o ciclo completo somente em
treinamento, com NUP/resíduo e contagem real de requests documentados.
