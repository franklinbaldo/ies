---
type: Guide
title: Credenciais públicas das instalações de treinamento
description: Usuário, senha e órgão publicados pelas próprias instalações de treinamento e homologação do SEI usadas pelo IES, com a fonte oficial de cada um
---

# Credenciais públicas de treinamento

São as contas que cada instalação de **treinamento ou homologação**
publica para uso aberto (fonte oficial na última coluna). Não são
credenciais de sessão: cookie, URL assinada e `infra_hash` continuam
proibidos em commits ([AGENTS.md](../../AGENTS.md)). A conta é
compartilhada com qualquer pessoa: o estado da tela (visão, página,
colunas, marcadores) muda sem aviso, e a senha pode mudar na fonte.
Divergência entre esta tabela e a fonte: vale a fonte.

| Chave | Instalação | Origem (`IES_ORIGEM_FILE`) | Usuário | Senha | Órgão | Fonte |
| --- | --- | --- | --- | --- | --- | --- |
| `unb` | UnB 4.0.9 | `https://sei.treinamento.unb.br/sei` | `aluno1` | `aluno1` | `UnB` | [fonte](https://portalsei.unb.br/ambiente-de-treinamento-do-sei-v-4-0/) |
| `pen1` | PEN1 5.0.4 (Tramita) | `https://sei.orgao1.tramita.processoeletronico.gov.br/sei` | `orgao1` | `orgao1` | `ORGAO1` | [fonte](https://wiki.processoeletronico.gov.br/pt-br/latest/Tramita_GOV_BR/Perguntas_frequentes/Ambientes_de_Homologacao_e_de%20_Producao.html) |
| `pen2` | PEN2 (Tramita) | `https://sei.orgao2.tramita.processoeletronico.gov.br/sei` | `orgao2` | `orgao2` | `ORGAO2` | [fonte](https://wiki.processoeletronico.gov.br/pt-br/latest/Tramita_GOV_BR/Perguntas_frequentes/Ambientes_de_Homologacao_e_de%20_Producao.html) |
| `pen3` | PEN3 5.0.4 (Tramita) | `https://sei.orgao3.tramita.processoeletronico.gov.br/sei` | `orgao3` | `orgao3` | `ORGAO` | [fonte](https://wiki.processoeletronico.gov.br/pt-br/latest/Tramita_GOV_BR/Perguntas_frequentes/Ambientes_de_Homologacao_e_de%20_Producao.html) |
| `pen4` | PEN4 5.0.4 (Tramita) | `https://sei.orgao4.tramita.processoeletronico.gov.br/sei` | `orgao4` | `orgao4` | `ORGAO4` | [fonte](https://wiki.processoeletronico.gov.br/pt-br/latest/Tramita_GOV_BR/Perguntas_frequentes/Ambientes_de_Homologacao_e_de%20_Producao.html) |
| `ufrj` | UFRJ (testes) | `https://teste.sei.tic.ufrj.br/sei` | `servidor1` | `servidor1` | `UFRJ` | [fonte](https://portal.sei.ufrj.br/capacitacao-e-orientacao/ambiente-de-testes/) |
| `go` | Goiás 5.0.3 | `https://seitreinamento.go.gov.br/sei` | `aluno05` | `abc123456` | `SEAD` | [fonte](https://sei.goias.gov.br/material_para_treinamento.php) |

Goiás tem WAF: rajadas recebem 403 "Acesso Negado"
(`bloqueio_de_seguranca_da_instalacao`); espaçar os comandos. A PEN2 fica
atrás do Cloudflare, que em 04/11/2026 bloqueou o login com 403 (mesmo
subcódigo).

## Arquivo de credenciais do `ies login`

O `ies login` só lê a senha de um arquivo privado (`0600`, sem link
simbólico) apontado por `IES_CREDENCIAIS_FILE`
([contrato](../../contracts/cli/sessao-login.md)). O arquivo de origem
leva a origem da tabela; cookie e URL do Controle podem nascer vazios
(`0600`), e o primeiro login os grava.

`unb`:

```text
usuario=aluno1
senha=aluno1
orgao=UnB
```

`pen1`:

```text
usuario=orgao1
senha=orgao1
orgao=ORGAO1
```

`pen2`:

```text
usuario=orgao2
senha=orgao2
orgao=ORGAO2
```

`pen3`:

```text
usuario=orgao3
senha=orgao3
orgao=ORGAO
```

`pen4`:

```text
usuario=orgao4
senha=orgao4
orgao=ORGAO4
```

`ufrj`:

```text
usuario=servidor1
senha=servidor1
orgao=UFRJ
```

`go`:

```text
usuario=aluno05
senha=abc123456
orgao=SEAD
```

