---
type: Guide
title: Acesso às interfaces por instalação do SEI
description: Disponibilidade REST e WSDL SOAP com autenticação e autorização verificadas
---

# Acesso REST, SOAP e Web por instalação

O [OpenAPI REST](rest-wssei.md) é uma especificação pública do módulo
WSSEI, **não um endpoint que necessariamente funciona na instalação**.
Há instalações em que o módulo não está disponível ou não funciona
para a conta/operação. O [WSDL SOAP](soap-sei.md) pode exigir
**credenciais e autorização até para ser consultado**; nunca
pressupor que o documento esteja acessível anonimamente.

O IES registra disponibilidade separadamente para cada combinação
**instalação + identidade/sistema + unidade + operação + protocolo**.
Uma instalação não recebe um único atributo global "suporta REST".

## Estados observáveis

| Estado | Quando atribuir | Consequência |
| --- | --- | --- |
| `unknown` | Não houve teste autorizado | Não escolher automaticamente |
| `auth-required` | Acesso depende de credenciais ainda não configuradas | Pedir configuração autorizada, sem sondar |
| `access-denied` | Serviço acessado com identidade autorizada, porém negou permissão | Não tentar trocar credencial ou backend silenciosamente |
| `unavailable-confirmed` | Ausência de módulo/serviço comprovada por evidência administrativa ou teste controlado inequívoco | Não selecionar esse protocolo |
| `available-confirmed` | Operação acessível com identidade e unidade autorizadas | Ainda exigir equivalência semântica |
| `incompatible` | Operação acessível, mas não cumpre o contrato de domínio | Recusar substituição automática |

**HTTP 401/403 não comprova ausência do módulo**. Um 404 tampouco
prova, sozinho, que REST ou SOAP inexista: pode haver URL errada,
roteamento reverso, exigência de autenticação ou mudança de instalação.
Não transformar uma falha de rede em resultado de capacidade.

## Condições antes de usar REST

1. Administrador ou documentação da instalação confirma módulo WSSEI e
   base HTTPS real — não copiar os servidores HTTP de exemplo do OpenAPI.
2. Existe **token autorizado para o módulo, identidade e unidade**.
   O token de REST não é automaticamente o cookie da interface Web nem
   a credencial de SOAP.
3. A operação foi testada e registrada, inclusive paginação, filtros,
   erros e cobertura dos campos exigidos pelo contrato do IES.
4. Nenhuma leitura de módulos restritos ou introspecção por tentativa
   aleatória de endpoints; não executar POST apenas para descobrir suporte.

## Condições antes de consultar o WSDL SOAP

1. Obter do administrador host, serviço externo cadastrado e **forma
   autorizada de autenticação para obter o WSDL**, se protegida.
2. Não tentar GET anônimo ou descobrir esquemas de autenticação por
   tentativa e erro onde o serviço exige credenciais.
3. Credenciais só em configuração privada/gerenciador de segredos.
   Nunca em URL, argumento CLI, frontmatter OKF, fixture, log, PR
   ou arquivo público.
4. Depois de autorizado, ler o WSDL e XSD **em ambiente controlado**,
   validar bindings, portas, endereço real, operações e permissões.
5. Separar "WSDL obtido" de **"método SOAP autorizado e executável"**:
   um não demonstra o outro.

## Escolha do backend

Se REST/SOAP não estiver configurado ou comprovadamente disponível,
o IES pode **considerar** o caminho Web já demonstrado, mas somente
quando ele satisfizer o mesmo contrato e a identidade/sessão estiver
autorizada. Isso não é autorização para fallback automático.

Para leituras, qualquer escolha alternativa precisa garantir a
semântica de domínio (mesmos conjuntos, identificadores e completude).
Para escritas, após timeout ou efeito incerto, **nunca** mudar backend
nem repetir a operação; primeiro reconciliar por leitura.

As [decisões por operação](../decisoes/unidade-listar-processos.md)
devem citar este procedimento e deixar estados desconhecidos como
desconhecidos. Nenhum registro das sete instalações do acervo foi
promovido a `available-confirmed` por esta política.
