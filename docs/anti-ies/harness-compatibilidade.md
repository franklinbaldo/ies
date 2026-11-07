---
type: Guide
title: Harness de compatibilidade e ablação IES ↔ anti-IES ↔ SEI
description: Testes diferenciais independentes e remoção gradual de HTML dispensável sem perder contratos observáveis
---

# Harness de compatibilidade

## Propósito e limites

O objetivo não é reproduzir todos os bytes do PHP/HTML do SEI.
O **IES** continua consumindo as instalações SEI reais; o
**anti-IES** serve respostas compatíveis que o IES e outros
consumidores relevantes conseguem interpretar; `iesei` compartilha
**domínio tipado e SQLite**, não o código de parsing/renderização.

**Não confundir concordância com verdade:** IES e anti-IES podem
compartilhar o mesmo erro. Os resultados esperados são definidos
por cassettes SEI verificadas, observações autorizadas e contratos
específicos, **antes** de comparar com o anti-IES.

O harness é **só de teste** (preferir `tests/compatibilidade/**` ou
testes de integração das crates existentes, sem nova crate ou daemon
por antecipação). Seu custo não entra no binário `iesei` de produção.
Usar os testes e gates canônicos do [AGENTS.md](../../AGENTS.md).

## Três caminhos independentes

| Via | Entrada | Resultado e propósito |
| --- | --- | --- |
| A — referência | Requisição/HTML/MIME do SEI preservados em cassette com hash conferido | O IES real reconhece a resposta autêntica e produz saída/recusa de referência |
| B — compatibilidade | Mesmo contexto e intenção, SQLite sintético alimentado pelo IES e resposta HTTP do **anti-IES executando de verdade em loopback** | O mesmo IES reconhece resposta gerada e mantém semântica, efeitos e erros equivalentes |
| C — confirmação externa | SEI de treinamento autorizado, quando for indispensável | Verifica lacunas empíricas; não entra em CI offline nem autoriza produção |

**Interoperabilidade do SEI oficial com artefatos produzidos** é uma
quarta prova independente, por formato/importação/serviços suportados,
quando houver essa capacidade. IES interpretar anti-IES **não prova**
que uma instalação oficial do SEI interpretará qualquer artefato.

Comparar estado HTTP/MIME, identidade, unidade, presença e cobertura
de lista, ordem quando significativa, erros tipados, transições de
estado e contagens GET/POST. Não exigir igualdade byte a byte, DOM
idêntico ou classes CSS sem importância comprovada.
Contar requests e verificar estado posterior em operações com efeito.

## Primeira fatia: `ies unidade atual`

Oráculo: [#353](https://github.com/franklinbaldo/ies/issues/353).
Modelo: [#360](https://github.com/franklinbaldo/ies/issues/360).
SQLite alimentado pelo IES: [#358](https://github.com/franklinbaldo/ies/issues/358).
Servidor Controle R/D: [#354](https://github.com/franklinbaldo/ies/issues/354).

O [harness #361](https://github.com/franklinbaldo/ies/issues/361)
deve cobrir pelo menos: Controle R e D, se observados; tela de
login com marcador isolado, tabela essencial ausente, unidade
divergente, estado desconhecido no SQLite. Um sucesso exige
**resposta HTTP real** do anti-IES, parser/caminho real do IES,
oráculo independente, **1 GET, 0 POST**, sem rede externa e sem
usar credenciais reais. Se for apenas parser com string, é teste
de componente, não primeira capacidade integrada.

As cassettes podem incluir formas distintas por instalação/versão.
Não fundir essas diferenças num HTML universal sem evidência.
O harness mantém casos positivos **e negativos** por forma conhecida.
`PASS/FAIL/NOT_RUN` deve indicar fonte, instalação/versão, SHA,
comandos e contagem observada de requests.

## Ablação progressiva: simplificar sem regredir

Depois da primeira integração E2E [#355](https://github.com/franklinbaldo/ies/issues/355),
executar uma ablação por vez [#362](https://github.com/franklinbaldo/ies/issues/362):

1. Formular hipótese falsificável: qual atributo, nó, wrapper ou
   peculiaridade HTTP é dispensável para **qual** operação/versão?
2. Remover ou modificar **uma característica** somente na resposta
   sintética do anti-IES. Não editar a cassette original nem alterar
   o esperado a partir do gerador.
3. Executar o harness A e B. Se passar, registrar teste
   **metamórfico**: a característica pode variar sem mudar o
   resultado semântico do cliente. Se falhar, preservar ou classificar
   a diferença como dependência real/possível.
4. Manter **mutação negativa**: retirar algo comprovadamente
   essencial (por exemplo tabela do Controle) deve continuar
   provocando erro correto. Evitar que o IES passe a aceitar
   qualquer HTML, incluindo uma página de login disfarçada.
5. Medir redução de bytes, parse, render, SQLite e RSS quando houver
   diferença material; performance não substitui a prova de
   compatibilidade.
6. Registrar `obrigatório/dispensável/não demonstrado` e suporte
   por instalação/versão, com SHA e fonte. **Não demonstrado não
   significa removível.**

Mesmo atributos aparentemente cosméticos podem ser contratuais:
ids, nomes de campos, hidden inputs, links assinados, seletores
e efeitos de GET/POST não são removíveis sem teste de consumidor.
O parser do IES só fica mais tolerante quando isso também é
aceitável para as **cassettes reais do SEI**. Não executar
JavaScript nem usar regex para interpretar HTML. Nenhum modo
de teste enfraquece TLS/origem/cookies da CLI real.

## Custo e integração contínua

Reusar `cargo test --locked --workspace --all-targets`, Rustfmt,
Clippy, OKF e o CI existente; inicialmente escolher uma suíte curta
de regressão de contrato por PR, sem baixar todo o acervo a cada
alteração. Replays completos ficam nas rotinas já existentes ou
rodadas específicas. Não criar segundo CI obrigatório ou depender
de rede real para todos os PRs. Benchmark de tempo/RSS separado de
gates funcionais sensíveis ao hardware.

A [#361](https://github.com/franklinbaldo/ies/issues/361) prepara
o harness; a [#355](https://github.com/franklinbaldo/ies/issues/355)
entrega seu primeiro uso E2E e marca INTEGRADA; a
[#362](https://github.com/franklinbaldo/ies/issues/362)
abre a simplificação progressiva **depois** da integração.
O [roadmap](roadmap.md) mantém as dependências e a
[matriz de cobertura](cobertura.md) registra a evidência.
