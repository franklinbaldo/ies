---
type: Guide
title: iesei — domínio Rust compartilhado
description: Fronteira inicial entre cliente IES, anti-IES e fatos observados
---

# iesei

`iesei` é a biblioteca de **tipos de domínio compartilhados**, sem
dependência de transporte HTML ou servidor HTTP. Os primeiros tipos
trazem `Instalacao`, unidade exibida, visões R/D do Controle,
faixa e cobertura observada e proveniência.

A biblioteca preserva uma diferença essencial: **não observado
não é vazio**. `Cobertura::NaoObservada` não equivale a uma
tabela efetivamente observada com zero registros. Os campos de
`ObservacaoDoControle` são informações de domínio, não cookies
ou URLs assinadas do SEI.

O IES coleta/valida respostas reais e alimentará o SQLite;
o anti-IES consultará projeções tipadas e renderizará respostas
sintéticas. O parser continua no cliente e o renderizador no
servidor; nenhum dos dois deve copiar as suposições do outro
para fabricar o oráculo.

Esta fatia da [#360](https://github.com/franklinbaldo/ies/issues/360)
migra a identidade de instalação do antigo `ies-core`.
A persistência SQLite é uma PR seguinte na pilha
([#358](https://github.com/franklinbaldo/ies/issues/358));
o primeiro teste E2E pertence à
[#355](https://github.com/franklinbaldo/ies/issues/355).
