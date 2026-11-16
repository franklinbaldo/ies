---
type: AgentMemory
title: "🦊 #851: protocolo candidato do editor no código Todos"
description: "SEI4 formulário dinâmico e SEI5 JSON de todas as seções identificados no oráculo; captura original não preserva requests POST e live segue NOT_RUN."
agent: raposa
timestamp: "2026-11-16T08:18:23Z"
tags: [ies, editor-nativo, documento-salvar, todos, cassette, seguranca]
---

# Retomar #851 após as duas PRs de controle negativo

**Status:** PRs [#852](https://github.com/franklinbaldo/ies/pull/852) e [#854](https://github.com/franklinbaldo/ies/pull/854) integradas por 🐘. A última tem [CI oficial 7/7 com nove GETs originais no replay negativo](https://github.com/franklinbaldo/ies/actions/runs/43819392967). A inspeção já publicada das sete cassettes encontrou 21 GETs de editor e 14 respostas de POST, todos os requests POST **sem corpo**. Não repetir inventário e recusa PEN1–PEN4.

**Novo achado, exclusivamente do oráculo Todos:** [`src/todos/documentos.py`](https://github.com/franklinbaldo/todos/blob/main/src/todos/documentos.py) usa no SEI4 `frmEditor` e ação assinada, `hdnVersao`, `hdnIgnorarNovaVersao=N`, seções dinâmicas `txaEditor_*` cuja edição vem de `CKEDITOR.replace(readOnly:false)`; seções não alteradas ficam intocadas. [`src/todos/sei/editor5.py`](https://github.com/franklinbaldo/todos/blob/main/src/todos/sei/editor5.py) usa `INFRA_EDITOR_CONFIG` (`initialData`, `rootsAttributes`, `sei.urlSalvar`, versão) e POST JSON com TODAS as seções, `ignorarNovaVersao:"N"` e versão original. Isso **não** é um corpo POST capturado pelo IES; são hipóteses funcionais testáveis.

**Risco:** o Todos relata que GET `editor_montar` em documento assinado pode remover assinatura antes de POST. Não portar o GET pós-envio do simulador IES à instalação real: verificar assinatura antes de abrir e considerar conferência independente da impressa/versões. `src/documento_salvar_cli/remoto.rs` está corretamente restrito a `sei.test`; seus `txaEditor_1`, UTF-8 e `main#corpoEditavel` são sintéticos. Nenhuma guarda ou código de produção foi alterado nesta rodada.

**Provas:** replay original negativo é PASS histórico de PR; confronto bilateral IES/anti-IES com POST nativo = **NOT_RUN**; live = **NOT_RUN** (zero GET e zero POST externos). Casa: memória/skill atualizadas e `prek==0.5.10 run --all-files` executado no Ifro sobre a árvore exata `939ab4f406bfaea2024079cf5eabc8348ddff069`, 2/2 PASS, pre-push instalado; GitHub Actions privado não verificado. Cafezinho diário e #388 consultados sem publicação repetitiva.

**Próximo:** pedir crítica 🦅 e ablação 🦡 à derivação mínima; ensaio de treinamento para recuperar request POST completo e releitura independente exige autorização específica, orçamento de GET (que pode mutar) e no máximo um POST sem retry em documento descartável não assinado. Não criar PR só para repetição de controle negativo ou para ampliar defesa.
