---
type: Guide
title: anti-IES — servidor experimental Rust dentro do IES
description: Escopo inicial, execução local e evolução por contratos observados do SEI
---

# anti-IES

O **anti-IES** é um servidor Rust no mesmo workspace que a CLI `ies`.
O objetivo é reproduzir progressivamente **efeitos e respostas observáveis**
do SEI Web, não clonar seu código PHP. A compatibilidade só existe para
operações explicitamente implementadas e verificadas.

## Executar

```sh
cargo run -p anti-ies -- 8765
curl -i http://127.0.0.1:8765/__anti_ies/health
curl -i 'http://127.0.0.1:8765/sei/controlador.php?acao=procedimento_trabalhar'
cargo test -p anti-ies
```

A primeira consulta retorna HTTP 200 e a identidade do servidor de testes.
A segunda retorna HTTP **501**: nenhuma página do SEI foi implementada.
Qualquer operação SEI não comprovada continua indisponível, sem simular
sucesso. A CLI `ies` existente permanece inalterada.

## Limites desta fatia

- Listener **somente** em `127.0.0.1`, sem configuração de interface externa.
- Transporte HTTP mínimo, sequencial e de desenvolvimento, que lê apenas a
  primeira linha da requisição (máximo 4096 bytes). **Não usar em produção**.
- Sem usuários, autenticação, cookies, banco de dados, JS, proxy ou acesso a
  instalações SEI; não serve como substituto do servidor oficial.
- Não há suporte a leitura de processos, documentos nem a escritas.
- Os testes são sintéticos e offline; validação live: **NOT_RUN**.

## Próxima fatia

1. Escolher uma operação Web somente leitura, observada em cassette do acervo
   e numa instalação de treinamento autorizada.
2. Descrever entradas, estado, erros e estrutura da resposta em contrato OKF;
   manter diferenças por instalação explícitas.
3. Construir modelo de domínio tipado e adaptador HTTP para **aquela**
   operação, com teste RED antes da implementação.
4. Usar o IES como consumidor dos dois lados e comparar resultados
   normalizados; identidade de HTML byte a byte não é pré-requisito.
5. Só quando houver contrato real, avaliar framework HTTP adequado e
   persistência, sem carregar o cliente com dependências de servidor.

Acompanhamento: [issue #350](https://github.com/franklinbaldo/ies/issues/350).

## Controle em leitura a partir do SQLite

Na bancada local, configurar um SQLite **sintético**, com as tabelas
`controle_observado`, `controle_tabela` e `controle_linha`
preenchidas pelo tipo `iesei::sqlite::registrar` (issue #358):

```sh
ANTI_IES_SQLITE=/privado/ensaio.sqlite3 \
ANTI_IES_ORIGEM=https://sei.exemplo/sei \
ANTI_IES_CONTEXTO=perfil-sintetico \
cargo run -p anti-ies -- 8765
```

O processo abre o arquivo somente leitura, sem coletar no SEI,
e responde à rota do Controle apenas quando os fatos armazenados
são suficientes. Ausência/estado parcial devolvem HTTP 503; rotas
SEI não implementadas continuam HTTP 501. **Não utilizar com dados
reais de sessão nem expor fora do loopback.** A fidelidade de
`unidade.atual` pelo caminho real do IES ainda requer o
harness E2E da issue #355.
