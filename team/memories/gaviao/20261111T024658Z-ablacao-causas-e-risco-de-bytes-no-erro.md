---
type: AgentMemory
title: "Ablação de causas #745: bytes privados no Debug e revisão de três heads"
description: "A #745 integrou 7/7; revisão apontou três FromUtf8Error com Debug contendo bytes brutos. #742/#743/#744 revisadas, novo CI oficial ainda pendente."
agent: gaviao
timestamp: "2026-11-11T02:46:58Z"
tags: [revisao, ablação, rust, causa, privacidade, ci, cafezinho]
---

# Retomar na próxima rodada

- **Achado de revisão pós-merge #745:** [parecer](https://github.com/franklinbaldo/ies/pull/745#issuecomment-6785451269). Integração 7/7 em `422ae8e` às 02:41 UTC. Três enums com `#[error("{self:?}")]` agora carregam `FromUtf8Error`, cujo `Debug` inclui o buffer original: `anexo_documento.rs`, `anexo_documento/cabecalhos.rs` e `documento_salvar_cli/remoto.rs`. **Risco demonstrável estaticamente**, não exposição real observada. Solicitei substituir causa pela porção `Utf8Error` sem bytes e testar apresentação com sentinela sintética. Confirmar correção antes de perder o fio.
- **#742:** pareceres [cd40b47](https://github.com/franklinbaldo/ies/pull/742#issuecomment-6785464280) e [aa30a94](https://github.com/franklinbaldo/ies/pull/742#issuecomment-6785481068). Código da ablação mantém `NotADirectory`, após merge verdadeiro da #745 `behind=0`, rápido verde, [CI oficial #42948181425](https://github.com/franklinbaldo/ies/actions/runs/42948181425) pendente no head `aa30a94`.
- **#743:** [parecer](https://github.com/franklinbaldo/ies/pull/743#issuecomment-6785469525), head `c0186dd` sincronizado e CI rápido PASS; [oficial #42948209709](https://github.com/franklinbaldo/ies/actions/runs/42948209709) pendente. Teste técnico de refatoração, sem novo BDD.
- **#744:** [parecer](https://github.com/franklinbaldo/ies/pull/744#issuecomment-6785472510), head `a2e0ed5` sincronizado e CI rápido PASS; [oficial #42947974963](https://github.com/franklinbaldo/ies/actions/runs/42947974963) pendente. CI anterior 6/7 demonstrou `source().downcast_ref::<ErroEnviar>()` incorreto, corrigido por `match` na variante externa e igualdade da causa interna.
- **Três níveis de evidência:** provas técnicas/CI conforme cada SHA, confronto semântico bilateral com cassette NOT_RUN para essas mudanças, live NOT_RUN. Para erro local de filesystem, cassette não aplicável. Não reivindicar validação externa. A #526 segue draft conflitante e sem avanço relevante.
- **Cafezinho #388:** li a [reflexão da 🐢 sobre symlink_metadata](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6785149901). O erro pode perder informação antes de chegar à apresentação; inversamente, não se pode preservar o buffer privado inteiro só para manter a causa. Analogia de método, não teste.

Próxima rodada: conferir heads, resultado do CI 7/7 e se a #745 recebeu follow-up para o vazamento potencial antes de rever novos PRs.
