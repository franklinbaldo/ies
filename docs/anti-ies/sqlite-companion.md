---
type: Guide
title: SQLite companion — IES produz, anti-IES consome
description: Modelo de observações compartilhadas entre o cliente IES e o servidor anti-IES
---

# SQLite companion: fluxo de dados

**IES é o produtor.** Ele consulta o SEI real somente nos ambientes
autorizados, usa o parser existente e registra as entidades, fatos,
relações e listas **efetivamente observados** no SQLite. Nenhum GET ou
POST extra é feito apenas para alimentar o banco.

**O `iesei` é a biblioteca comum.** IES e anti-IES importam
os mesmos modelos e as operações tipadas de SQLite. IES escreve
observações depois de ler e validar o SEI; anti-IES somente
consulta a projeção autorizada. A [#360](https://github.com/franklinbaldo/ies/issues/360)
consolida o atual `ies-core` em `iesei`, sem trazer parsers
HTML, clientes HTTP, CLI ou templates do servidor para o crate.
Ver [modelo](modelo-entidades.md).

**SQLite é memória, não espelho completo.** O `ies.sqlite3` já contém
identidades de processo/documento, unidades e links assinados. Evoluir
esse índice e suas migrações para uma projeção tipada de observações,
com instalação, contexto, proveniência, momento da observação e
completude (página/lista observada ou inteira).

**anti-IES é consumidor.** O servidor lê somente essa projeção,
gera respostas HTTP/HTML compatíveis com os cenários observados e
não acessa o SEI nem cria fatos no companion. Dados ausentes do SQLite
são **desconhecidos**, não automaticamente inexistentes no SEI.
Nunca produzir tabelas de zero registros sem observação comprovada.

O banco do IES contém `link_assinado`, que pode trazer `infra_hash`.
O anti-IES não deve ler ou publicar essas tabelas, cookies, URLs
assinadas ou credenciais. Na primeira etapa, usar somente banco
sintético privado, modo read-only e loopback. Exposição futura exige
uma projeção SQLite sanitizada separada ou isolamento comprovado.
Não compartilhar fatos entre usuários ou unidades sem contexto
de identidade demonstrado. Falha de gravação do índice não deve
derrubar a leitura Web do IES.

O [modelo incremental de entidades](modelo-entidades.md) especifica
identidades, observações, grau de cobertura e desempenho. A modelagem
mínima [#360](https://github.com/franklinbaldo/ies/issues/360) vem
**antes** da persistência da primeira observação na #358.

Para escrita *simulada* futura, o anti-IES usará estado de bancada
separado dos fatos históricos coletados pelo IES. Não extrair crate
nova ou reescrever SQLite por antecipação: aproveitar `rusqlite`,
WAL e migrações existentes conforme
[ADR 0003](../arquitetura/0003-indice-local.md) e [#188](https://github.com/franklinbaldo/ies/issues/188).

## Primeira capacidade integrada

1. [P1 #353](https://github.com/franklinbaldo/ies/issues/353):
   contrato e oráculo independentes para `unidade.atual`.
2. [P1.2 #360](https://github.com/franklinbaldo/ies/issues/360):
   modelo Rust tipado e plano de schema/índices com medição de base.
3. [P1.5 #358](https://github.com/franklinbaldo/ies/issues/358):
   o IES grava unidade/visão/Controle observados no SQLite.
4. [P2 #354](https://github.com/franklinbaldo/ies/issues/354):
   o anti-IES lê a projeção e serve o Controle por HTTP.
5. [P3 #355](https://github.com/franklinbaldo/ies/issues/355):
   teste E2E **IES → SQLite → anti-IES → IES** comparado
   ao SEI/cassette, inclusive erros e contagem de requests.

Nenhum marco é concluído sem CI verde do SHA e evidência do
caminho real. Ver [roadmap conjunto](roadmap.md) e
[matriz de cobertura](cobertura.md).
