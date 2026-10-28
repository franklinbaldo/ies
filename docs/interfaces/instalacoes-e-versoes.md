---
type: Guide
title: Instalação SEI, órgão, unidade e versões
description: Separar ambiente técnico, estrutura organizacional e compatibilidade demonstrada
---

# Instalação, órgão e unidade

Uma **instalação** é o ambiente técnico do SEI, identificado para
conexão pelo endereço-base HTTPS (host e caminho-base). A mesma
instalação pode atender vários órgãos; um host também pode hospedar
instalações em caminhos distintos. O endereço não é identidade de órgão.

Um **órgão** é uma instituição configurada no SIP/SEI. Uma **unidade**
pertence à estrutura de um órgão, com suas permissões e hierarquia.
A unidade ativa é parte do contexto da sessão autenticada e pode
mudar sem mudar a instalação.

O elemento `lnkInfraUnidade` das cassettes indica apenas o **rótulo
visual da unidade ativa**. Não deduzir dele o ID interno da unidade,
o órgão de cadastro ou uma identidade criptográfica de sessão.

A distinção consta dos manuais oficiais sobre
[criação de órgãos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Configura%C3%A7%C3%A3o_da_estrutura/Man_Adm_SEI_03-Config_Estr_Organizacional-01_Criacao_de_Orgaos.html)
e [configuração de unidades](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Configura%C3%A7%C3%A3o_da_estrutura/Man_Adm_SEI_03-Config_Estr_Organizacional-021_Configuracao_de_Unidades.html).

## Tipo Rust mínimo

`Instalacao` representa endereço-base confiável e a versão SEI
opcionalmente **declarada**. Não contém cookies, credenciais,
usuário, órgão ou unidade.

`VersaoSei` interpreta versões de duas a quatro componentes
numéricos, como `4.0.9`, `4.1.5`, `5.0.3`, `5.0.4`. Não
pressupõe SemVer, nem associa automaticamente a versão a recursos.

O arquivo privado `--origem-file` informa a instalação:

```text
https://sei.exemplo.gov.br/sei
versao=5.0.4
```

A segunda linha é opcional. O caminho-base é importante: cookies
somente são enviados quando **host e caminho do controlador**
correspondem à instalação autorizada. A versão declarada nunca
amplia essa autorização. Arquivos antigos com apenas `https://host`
representam uma instalação no caminho raiz; para instalações
em `/sei`, é preciso registrar o caminho explicitamente.

## Confiança de compatibilidade por operação

Compatibilidade não é atributo booleano da versão nem da instalação.
É uma conclusão **a partir de testes e condições concretas**.

| Situação | Significado | Uso permitido |
| --- | --- | --- |
| Verificada na instalação | Operação passou no ambiente, versão, identidade e contexto avaliados | Sustenta aquele recorte de operação |
| Sugerida por mesma versão | Passou noutra instalação da mesma versão | Priorizar testes; não executar automaticamente |
| Desconhecida | Sem evidência relevante | Não presumir suporte |
| Divergência reproduzida | Um teste mostrou comportamento incompatível | Registrar e impedir substituição sem correção |

Não atribuir percentuais fictícios ou salvar campos `confianca`
como verdade no modelo. A versão sozinha não comprova autenticação,
permissão, módulo REST instalado, WSDL acessível nem equivalência.
Customizações podem mudar o comportamento dentro da mesma versão.

## Recorte já reproduzido

Em outubro de 2026 foram documentadas as seguintes versões de
instalações de treinamento; os sete replays da **caixa Web** foram
verificados, não o conjunto inteiro de operações do SEI:

| Instalação | Versão registrada | Evidência |
| --- | --- | --- |
| UnB | 4.0.9 | Replay de caixa |
| UFRJ | 4.1.5 | Replay de caixa |
| Goiás | 5.0.3 | Replay de caixa |
| PEN1 | 4.0.12 | Replay de caixa |
| PEN2 | 4.1.5 | Replay de caixa |
| PEN3 | 5.0.4 | Replay de caixa |
| PEN4 | 5.0.4 | Replay de caixa |

Esses valores são registros históricos, não detecção live. Para
cada nova funcionalidade, usar o contrato da CLI, os testes Rust e
as cassettes/OKF aplicáveis. Testar em instalações diferentes da
mesma versão ajuda a distinguir versão de configuração local.

O cliente Web continua necessário; REST e SOAP são alternativas
quando disponíveis, autorizadas e equivalentes. Nenhum suporte
é concedido automaticamente por uma `VersaoSei`.

Consulte [estado e instalações](../como-o-sei-funciona/estado-e-instalacoes.md),
[política de acesso por instalação](acesso-por-instalacao.md) e
[decisões por operação](../decisoes/unidade-listar-processos.md).
